//! The kind `kierownik-biura-przydzial` of the room `positions/biuro/kierownik_biura`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind's name.
pub const KIND: &str = "kierownik-biura-przydzial";

/// The migrations of the kind, in the order of their numbers.
pub const MIGRATIONS: &[&str] = &[
    r#"-- Przydział nowych spraw: reguły, który rodzaj zgłoszenia trafia do którego działu, i przydziały
-- kart Recepcji z wynikiem sprawdzenia w kartotece klientów.
--
-- Regułę zatwierdza właściciel biura podpisem w wiadomości; kierownik przenosi podpis do wiersza,
-- a sprawdza go aplikacja. Zmiana reguły to nowa reguła, a poprzednia dostaje datę zastąpienia,
-- więc historia przydziałów zostaje (R9). Na jeden rodzaj zgłoszenia jest naraz jedna reguła.
CREATE TABLE kierownik_biura_przydzial_reguly (
  -- Klucz nadany przez bazę jest dodatni; zob. kierownik_biura_przydzial_reguly_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  -- Czego chce zgłaszający, jak na karcie Recepcji.
  rodzaj_zgloszenia TEXT NOT NULL CHECK (rodzaj_zgloszenia IN ('sprzedaz', 'kupno', 'wynajem', 'zarzad')),
  dzial TEXT NOT NULL CHECK (dzial IN ('posrednictwo', 'transakcje', 'najem')),
  od TEXT NOT NULL,
  podpisal TEXT NOT NULL CHECK (podpisal <> ''),
  podpis TEXT NOT NULL CHECK (podpis <> ''),
  zastapiona_od TEXT,

  CONSTRAINT zastapienie_nie_przed_poczatkiem CHECK (zastapiona_od IS NULL OR zastapiona_od >= od)
) STRICT;

-- Karta Recepcji i to, jak kierownik ją rozstrzygnął. Pola karty pochodzą od zgłaszającego, więc są
-- zgłoszeniem, a nie faktem. Kontakt leży na karcie tylko do rozstrzygnięcia: po przydziale trafił
-- do działu, a po odrzuceniu jest zbędny. Wynik sprawdzenia w kartotece klientów to też ślad
-- odczytu kartoteki z powodem.
--
-- Przydział, którego reguły nie rozstrzygają, i odrzucenie zgłoszenia klienta wymagają zgody
-- człowieka; jego podpis kierownik przenosi do wiersza. Na początku (okres wdrożenia) człowiek
-- zatwierdza też przydziały według reguł.
CREATE TABLE kierownik_biura_przydzial_karty (
  -- Klucz nadany przez bazę jest dodatni; zob. kierownik_biura_przydzial_karty_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  -- Karta w pamięci Recepcji, tekstem.
  karta_odnosnik TEXT NOT NULL UNIQUE CHECK (karta_odnosnik <> ''),
  wplynela TEXT NOT NULL,
  rodzaj_karty TEXT NOT NULL CHECK (rodzaj_karty IN ('nowe_zgloszenie', 'pismo_bez_sprawy', 'skarga')),
  rodzaj_sprawy TEXT CHECK (rodzaj_sprawy IN ('sprzedaz', 'kupno', 'wynajem', 'zarzad')),
  rodzaj_nieruchomosci TEXT,
  miejscowosc TEXT,
  kanal TEXT,
  kontakt TEXT,
  -- Czy zgłaszający jest w kartotece klientów: nie ma go, jest jeden wpis albo dane pasują do kilku
  -- wpisów lub tylko częściowo, a wtedy kierownik pisze to kierownikowi działu i nie wybiera sam.
  kartoteka_wynik TEXT CHECK (kartoteka_wynik IN ('brak', 'jeden', 'kilka_albo_czesciowo')),
  kartoteka_odnosnik TEXT,
  sprawdzono_kartoteke TEXT,
  stan TEXT NOT NULL CHECK (stan IN ('czeka', 'u_czlowieka', 'przydzielona', 'odrzucona')),
  dzial TEXT CHECK (dzial IN ('posrednictwo', 'transakcje', 'najem')),
  regula_id INTEGER REFERENCES kierownik_biura_przydzial_reguly (id),
  powod TEXT,
  rozstrzygnieto TEXT,
  zatwierdzil TEXT,
  podpis TEXT,
  zatwierdzono TEXT,

  CONSTRAINT sprawdzenie_kartoteki_w_komplecie CHECK (
    (kartoteka_wynik IS NULL) = (sprawdzono_kartoteke IS NULL)
    AND (kartoteka_wynik IS 'jeden') = (kartoteka_odnosnik IS NOT NULL)
  ),
  CONSTRAINT przydzielona_z_dzialem CHECK ((stan = 'przydzielona') = (dzial IS NOT NULL)),
  CONSTRAINT odrzucona_z_powodem CHECK (stan <> 'odrzucona' OR powod IS NOT NULL),
  CONSTRAINT rozstrzygnieta_z_data CHECK ((stan IN ('przydzielona', 'odrzucona')) = (rozstrzygnieto IS NOT NULL)),
  CONSTRAINT rozstrzygnieta_bez_kontaktu CHECK (stan NOT IN ('przydzielona', 'odrzucona') OR kontakt IS NULL),
  CONSTRAINT przydzielone_zgloszenie_sprawdzone_w_kartotece CHECK (
    stan <> 'przydzielona'
    OR rodzaj_karty <> 'nowe_zgloszenie'
    OR kartoteka_wynik IS NOT NULL
  ),
  CONSTRAINT przydzial_poza_regulami_za_zgoda_czlowieka CHECK (
    stan <> 'przydzielona'
    OR regula_id IS NOT NULL
    OR zatwierdzil IS NOT NULL
  ),
  CONSTRAINT odrzucenie_zgloszenia_za_zgoda_czlowieka CHECK (
    stan <> 'odrzucona'
    OR rodzaj_karty <> 'nowe_zgloszenie'
    OR zatwierdzil IS NOT NULL
  ),
  CONSTRAINT zatwierdzenie_w_komplecie CHECK (
    (zatwierdzil IS NULL) = (podpis IS NULL)
    AND (zatwierdzil IS NULL) = (zatwierdzono IS NULL)
  ),
  CONSTRAINT rozstrzygniecie_nie_przed_wplywem CHECK (rozstrzygnieto IS NULL OR rozstrzygnieto >= wplynela)
) STRICT;

-- Na jeden rodzaj zgłoszenia jest naraz jedna reguła; nowa przychodzi po zastąpieniu starej.
CREATE UNIQUE INDEX kierownik_biura_przydzial_jedna_regula_na_rodzaj
ON kierownik_biura_przydzial_reguly (rodzaj_zgloszenia)
WHERE zastapiona_od IS NULL;

-- Reguły -----------------------------------------------------------------------------------------

CREATE TRIGGER kierownik_biura_przydzial_regula_powstaje_aktualna
BEFORE INSERT ON kierownik_biura_przydzial_reguly
WHEN NEW.zastapiona_od IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'reguła przydziału powstaje aktualna; zastąpienie odnotowuje się później');
END;

-- Druga aktualna reguła z innym kluczem zderzyłaby się z indeksem, a INSERT OR REPLACE usunąłby
-- wtedy pierwszą z ominięciem blokady usuwania; ten trigger odmawia wcześniej.
CREATE TRIGGER kierownik_biura_przydzial_jedna_regula_przed_zastapieniem
BEFORE INSERT ON kierownik_biura_przydzial_reguly
WHEN EXISTS (
  SELECT 1
  FROM kierownik_biura_przydzial_reguly
  WHERE rodzaj_zgloszenia = NEW.rodzaj_zgloszenia
    AND zastapiona_od IS NULL
    AND id IS NOT NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'ten rodzaj zgłoszenia ma już aktualną regułę; nowa przychodzi po zastąpieniu starej');
END;

-- Podpis obejmuje regułę, więc zmienia się tylko raz odnotowane zastąpienie.
CREATE TRIGGER kierownik_biura_przydzial_regula_bez_zmian
BEFORE UPDATE ON kierownik_biura_przydzial_reguly
WHEN OLD.zastapiona_od IS NOT NULL
  OR NEW.id IS NOT OLD.id
  OR NEW.rodzaj_zgloszenia IS NOT OLD.rodzaj_zgloszenia
  OR NEW.dzial IS NOT OLD.dzial
  OR NEW.od IS NOT OLD.od
  OR NEW.podpisal IS NOT OLD.podpisal
  OR NEW.podpis IS NOT OLD.podpis
BEGIN
  SELECT RAISE(ABORT, 'reguły przydziału się nie zmienia; zmiana to nowa reguła, a stara dostaje datę zastąpienia');
END;

-- Karty ------------------------------------------------------------------------------------------

CREATE TRIGGER kierownik_biura_przydzial_karta_powstaje_czekajaca
BEFORE INSERT ON kierownik_biura_przydzial_karty
WHEN NEW.stan <> 'czeka'
BEGIN
  SELECT RAISE(ABORT, 'karta powstaje jako czekająca; rozstrzygnięcie przychodzi później');
END;

-- Przydział według reguły idzie do działu tej reguły dla tego rodzaju sprawy, a reguła była
-- aktualna, gdy karta wpłynęła.
CREATE TRIGGER kierownik_biura_przydzial_zgodny_z_regula
BEFORE UPDATE OF regula_id, dzial ON kierownik_biura_przydzial_karty
WHEN NEW.regula_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM kierownik_biura_przydzial_reguly AS regula
    WHERE regula.id = NEW.regula_id
      AND regula.dzial = NEW.dzial
      AND regula.rodzaj_zgloszenia IS NEW.rodzaj_sprawy
      AND regula.od <= NEW.wplynela
      AND (regula.zastapiona_od IS NULL OR regula.zastapiona_od > NEW.wplynela)
  )
BEGIN
  SELECT RAISE(ABORT, 'przydział według reguły idzie do jej działu, dla jej rodzaju zgłoszenia, gdy była aktualna');
END;

-- Pola karty są zgłoszeniem z Recepcji; rozstrzygniętej karty nie zmienia się wcale.
CREATE TRIGGER kierownik_biura_przydzial_karta_bez_zmian
BEFORE UPDATE ON kierownik_biura_przydzial_karty
WHEN OLD.stan IN ('przydzielona', 'odrzucona')
  OR NEW.id IS NOT OLD.id
  OR NEW.karta_odnosnik IS NOT OLD.karta_odnosnik
  OR NEW.wplynela IS NOT OLD.wplynela
  OR NEW.rodzaj_karty IS NOT OLD.rodzaj_karty
  OR NEW.rodzaj_sprawy IS NOT OLD.rodzaj_sprawy
  OR NEW.rodzaj_nieruchomosci IS NOT OLD.rodzaj_nieruchomosci
  OR NEW.miejscowosc IS NOT OLD.miejscowosc
  OR NEW.kanal IS NOT OLD.kanal
  OR (OLD.kartoteka_wynik IS NOT NULL AND (
    NEW.kartoteka_wynik IS NOT OLD.kartoteka_wynik
    OR NEW.kartoteka_odnosnik IS NOT OLD.kartoteka_odnosnik
    OR NEW.sprawdzono_kartoteke IS NOT OLD.sprawdzono_kartoteke
  ))
BEGIN
  SELECT RAISE(ABORT, 'pól karty ani sprawdzenia w kartotece się nie zmienia, a rozstrzygniętej karty wcale');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers; także kartę o tym samym
-- odnośniku. Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo
-- klucze są dodatnie.
CREATE TRIGGER kierownik_biura_przydzial_reguly_bez_zastepowania
BEFORE INSERT ON kierownik_biura_przydzial_reguly
WHEN EXISTS (
  SELECT 1
  FROM kierownik_biura_przydzial_reguly
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

CREATE TRIGGER kierownik_biura_przydzial_karty_bez_zastepowania
BEFORE INSERT ON kierownik_biura_przydzial_karty
WHEN EXISTS (
  SELECT 1
  FROM kierownik_biura_przydzial_karty
  WHERE id = NEW.id
    OR karta_odnosnik = NEW.karta_odnosnik
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- R9: zastąpiona reguła i rozstrzygnięta karta zostają jako historia przydziałów.
CREATE TRIGGER kierownik_biura_przydzial_reguly_bez_usuwania
BEFORE DELETE ON kierownik_biura_przydzial_reguly
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;

CREATE TRIGGER kierownik_biura_przydzial_karty_bez_usuwania
BEFORE DELETE ON kierownik_biura_przydzial_karty
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;
"#,
];

/// The values of column `karty.rodzaj_karty`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KartyRodzajKarty {
    NoweZgloszenie,
    PismoBezSprawy,
    Skarga,
}

impl KartyRodzajKarty {
    /// Every value, in the order of the catalog.
    pub const ALL: &[KartyRodzajKarty] = &[
        KartyRodzajKarty::NoweZgloszenie,
        KartyRodzajKarty::PismoBezSprawy,
        KartyRodzajKarty::Skarga,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            KartyRodzajKarty::NoweZgloszenie => "nowe_zgloszenie",
            KartyRodzajKarty::PismoBezSprawy => "pismo_bez_sprawy",
            KartyRodzajKarty::Skarga => "skarga",
        }
    }
}

impl xmeta_room::FromSqlValue for KartyRodzajKarty {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "nowe_zgloszenie" => Ok(KartyRodzajKarty::NoweZgloszenie),
                    "pismo_bez_sprawy" => Ok(KartyRodzajKarty::PismoBezSprawy),
                    "skarga" => Ok(KartyRodzajKarty::Skarga),
                    other => Err(format!("{other} is no value of KartyRodzajKarty")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for KartyRodzajKarty {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `karty.rodzaj_sprawy`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KartyRodzajSprawy {
    Sprzedaz,
    Kupno,
    Wynajem,
    Zarzad,
}

impl KartyRodzajSprawy {
    /// Every value, in the order of the catalog.
    pub const ALL: &[KartyRodzajSprawy] = &[
        KartyRodzajSprawy::Sprzedaz,
        KartyRodzajSprawy::Kupno,
        KartyRodzajSprawy::Wynajem,
        KartyRodzajSprawy::Zarzad,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            KartyRodzajSprawy::Sprzedaz => "sprzedaz",
            KartyRodzajSprawy::Kupno => "kupno",
            KartyRodzajSprawy::Wynajem => "wynajem",
            KartyRodzajSprawy::Zarzad => "zarzad",
        }
    }
}

impl xmeta_room::FromSqlValue for KartyRodzajSprawy {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "sprzedaz" => Ok(KartyRodzajSprawy::Sprzedaz),
                    "kupno" => Ok(KartyRodzajSprawy::Kupno),
                    "wynajem" => Ok(KartyRodzajSprawy::Wynajem),
                    "zarzad" => Ok(KartyRodzajSprawy::Zarzad),
                    other => Err(format!("{other} is no value of KartyRodzajSprawy")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for KartyRodzajSprawy {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `karty.kartoteka_wynik`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KartyKartotekaWynik {
    Brak,
    Jeden,
    KilkaAlboCzesciowo,
}

impl KartyKartotekaWynik {
    /// Every value, in the order of the catalog.
    pub const ALL: &[KartyKartotekaWynik] = &[
        KartyKartotekaWynik::Brak,
        KartyKartotekaWynik::Jeden,
        KartyKartotekaWynik::KilkaAlboCzesciowo,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            KartyKartotekaWynik::Brak => "brak",
            KartyKartotekaWynik::Jeden => "jeden",
            KartyKartotekaWynik::KilkaAlboCzesciowo => "kilka_albo_czesciowo",
        }
    }
}

impl xmeta_room::FromSqlValue for KartyKartotekaWynik {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "brak" => Ok(KartyKartotekaWynik::Brak),
                    "jeden" => Ok(KartyKartotekaWynik::Jeden),
                    "kilka_albo_czesciowo" => Ok(KartyKartotekaWynik::KilkaAlboCzesciowo),
                    other => Err(format!("{other} is no value of KartyKartotekaWynik")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for KartyKartotekaWynik {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `karty.stan`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KartyStan {
    Czeka,
    UCzlowieka,
    Przydzielona,
    Odrzucona,
}

impl KartyStan {
    /// Every value, in the order of the catalog.
    pub const ALL: &[KartyStan] = &[
        KartyStan::Czeka,
        KartyStan::UCzlowieka,
        KartyStan::Przydzielona,
        KartyStan::Odrzucona,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            KartyStan::Czeka => "czeka",
            KartyStan::UCzlowieka => "u_czlowieka",
            KartyStan::Przydzielona => "przydzielona",
            KartyStan::Odrzucona => "odrzucona",
        }
    }
}

impl xmeta_room::FromSqlValue for KartyStan {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "czeka" => Ok(KartyStan::Czeka),
                    "u_czlowieka" => Ok(KartyStan::UCzlowieka),
                    "przydzielona" => Ok(KartyStan::Przydzielona),
                    "odrzucona" => Ok(KartyStan::Odrzucona),
                    other => Err(format!("{other} is no value of KartyStan")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for KartyStan {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `karty.dzial`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KartyDzial {
    Posrednictwo,
    Transakcje,
    Najem,
}

impl KartyDzial {
    /// Every value, in the order of the catalog.
    pub const ALL: &[KartyDzial] = &[
        KartyDzial::Posrednictwo,
        KartyDzial::Transakcje,
        KartyDzial::Najem,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            KartyDzial::Posrednictwo => "posrednictwo",
            KartyDzial::Transakcje => "transakcje",
            KartyDzial::Najem => "najem",
        }
    }
}

impl xmeta_room::FromSqlValue for KartyDzial {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "posrednictwo" => Ok(KartyDzial::Posrednictwo),
                    "transakcje" => Ok(KartyDzial::Transakcje),
                    "najem" => Ok(KartyDzial::Najem),
                    other => Err(format!("{other} is no value of KartyDzial")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for KartyDzial {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// A row of `karty_do_rozstrzygniecia`.
#[derive(Debug, Clone, PartialEq)]
pub struct KartyDoRozstrzygnieciaRow {
    pub id: i64,
    pub karta_odnosnik: String,
    pub wplynela: String,
    pub rodzaj_karty: KartyRodzajKarty,
    pub rodzaj_sprawy: Option<KartyRodzajSprawy>,
    pub rodzaj_nieruchomosci: Option<String>,
    pub miejscowosc: Option<String>,
    pub kanal: Option<String>,
    pub kontakt: Option<String>,
    pub kartoteka_wynik: Option<KartyKartotekaWynik>,
    pub kartoteka_odnosnik: Option<String>,
    pub sprawdzono_kartoteke: Option<String>,
    pub stan: KartyStan,
    pub dzial: Option<KartyDzial>,
    pub regula_id: Option<i64>,
    pub powod: Option<String>,
    pub rozstrzygnieto: Option<String>,
    pub zatwierdzil: Option<String>,
    pub podpis: Option<String>,
    pub zatwierdzono: Option<String>,
}

impl KartyDoRozstrzygnieciaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(KartyDoRozstrzygnieciaRow {
            id: row.get("id")?,
            karta_odnosnik: row.get("karta_odnosnik")?,
            wplynela: row.get("wplynela")?,
            rodzaj_karty: row.get("rodzaj_karty")?,
            rodzaj_sprawy: row.get("rodzaj_sprawy")?,
            rodzaj_nieruchomosci: row.get("rodzaj_nieruchomosci")?,
            miejscowosc: row.get("miejscowosc")?,
            kanal: row.get("kanal")?,
            kontakt: row.get("kontakt")?,
            kartoteka_wynik: row.get("kartoteka_wynik")?,
            kartoteka_odnosnik: row.get("kartoteka_odnosnik")?,
            sprawdzono_kartoteke: row.get("sprawdzono_kartoteke")?,
            stan: row.get("stan")?,
            dzial: row.get("dzial")?,
            regula_id: row.get("regula_id")?,
            powod: row.get("powod")?,
            rozstrzygnieto: row.get("rozstrzygnieto")?,
            zatwierdzil: row.get("zatwierdzil")?,
            podpis: row.get("podpis")?,
            zatwierdzono: row.get("zatwierdzono")?,
        })
    }
}

/// Karty, które czekają na kierownika albo na zgodę człowieka, od najstarszej.
pub fn karty_do_rozstrzygniecia(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<KartyDoRozstrzygnieciaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM kierownik_biura_przydzial_karty
WHERE stan IN ('czeka', 'u_czlowieka')
ORDER BY wplynela, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(KartyDoRozstrzygnieciaRow::from_row).collect()
}

/// The parameters of `karty_rozstrzygniete_od`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct KartyRozstrzygnieteOdParams {
    pub od: String,
}

/// A row of `karty_rozstrzygniete_od`.
#[derive(Debug, Clone, PartialEq)]
pub struct KartyRozstrzygnieteOdRow {
    pub id: i64,
    pub karta_odnosnik: String,
    pub wplynela: String,
    pub rodzaj_karty: KartyRodzajKarty,
    pub rodzaj_sprawy: Option<KartyRodzajSprawy>,
    pub rodzaj_nieruchomosci: Option<String>,
    pub miejscowosc: Option<String>,
    pub kanal: Option<String>,
    pub kontakt: Option<String>,
    pub kartoteka_wynik: Option<KartyKartotekaWynik>,
    pub kartoteka_odnosnik: Option<String>,
    pub sprawdzono_kartoteke: Option<String>,
    pub stan: KartyStan,
    pub dzial: Option<KartyDzial>,
    pub regula_id: Option<i64>,
    pub powod: Option<String>,
    pub rozstrzygnieto: Option<String>,
    pub zatwierdzil: Option<String>,
    pub podpis: Option<String>,
    pub zatwierdzono: Option<String>,
}

impl KartyRozstrzygnieteOdRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(KartyRozstrzygnieteOdRow {
            id: row.get("id")?,
            karta_odnosnik: row.get("karta_odnosnik")?,
            wplynela: row.get("wplynela")?,
            rodzaj_karty: row.get("rodzaj_karty")?,
            rodzaj_sprawy: row.get("rodzaj_sprawy")?,
            rodzaj_nieruchomosci: row.get("rodzaj_nieruchomosci")?,
            miejscowosc: row.get("miejscowosc")?,
            kanal: row.get("kanal")?,
            kontakt: row.get("kontakt")?,
            kartoteka_wynik: row.get("kartoteka_wynik")?,
            kartoteka_odnosnik: row.get("kartoteka_odnosnik")?,
            sprawdzono_kartoteke: row.get("sprawdzono_kartoteke")?,
            stan: row.get("stan")?,
            dzial: row.get("dzial")?,
            regula_id: row.get("regula_id")?,
            powod: row.get("powod")?,
            rozstrzygnieto: row.get("rozstrzygnieto")?,
            zatwierdzil: row.get("zatwierdzil")?,
            podpis: row.get("podpis")?,
            zatwierdzono: row.get("zatwierdzono")?,
        })
    }
}

/// Karty przydzielone i odrzucone od jednego dnia, do sprawozdania i do porównania z regułami.
pub fn karty_rozstrzygniete_od(
    room: &impl xmeta_room::MetaRoom,
    params: &KartyRozstrzygnieteOdParams,
) -> Result<Vec<KartyRozstrzygnieteOdRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM kierownik_biura_przydzial_karty
WHERE stan IN ('przydzielona', 'odrzucona')
  AND rozstrzygnieto >= :od
ORDER BY rozstrzygnieto, id"#,
        parameters: Vec::from([("od", xmeta_room::ToSqlValue::to_sql_value(&params.od))]),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(KartyRozstrzygnieteOdRow::from_row).collect()
}

/// The parameters of `przyjmij_karte`.
#[derive(Debug, Clone, PartialEq)]
pub struct PrzyjmijKarteParams {
    pub karta_odnosnik: String,
    pub wplynela: String,
    pub rodzaj_karty: KartyRodzajKarty,
    pub rodzaj_sprawy: Option<KartyRodzajSprawy>,
    pub rodzaj_nieruchomosci: Option<String>,
    pub miejscowosc: Option<String>,
    pub kanal: Option<String>,
    pub kontakt: Option<String>,
}

/// Karta od Recepcji ze stałymi polami.
pub fn przyjmij_karte(
    room: &impl xmeta_room::MetaRoom,
    params: &PrzyjmijKarteParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO kierownik_biura_przydzial_karty (karta_odnosnik, wplynela, rodzaj_karty, rodzaj_sprawy,
  rodzaj_nieruchomosci, miejscowosc, kanal, kontakt, stan)
VALUES (:karta_odnosnik, :wplynela, :rodzaj_karty, :rodzaj_sprawy, :rodzaj_nieruchomosci,
  :miejscowosc, :kanal, :kontakt, 'czeka')"#,
                parameters: Vec::from([
                    (
                        "karta_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.karta_odnosnik),
                    ),
                    ("wplynela", xmeta_room::ToSqlValue::to_sql_value(&params.wplynela)),
                    (
                        "rodzaj_karty",
                        xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj_karty),
                    ),
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
                    ("kontakt", xmeta_room::ToSqlValue::to_sql_value(&params.kontakt)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_sprawdzenie_kartoteki`.
#[derive(Debug, Clone, PartialEq)]
pub struct OdnotujSprawdzenieKartotekiParams {
    pub kartoteka_wynik: KartyKartotekaWynik,
    pub kartoteka_odnosnik: Option<String>,
    pub sprawdzono_kartoteke: String,
    pub id: i64,
}

/// Wynik sprawdzenia zgłaszającego w kartotece klientów.
pub fn odnotuj_sprawdzenie_kartoteki(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujSprawdzenieKartotekiParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kierownik_biura_przydzial_karty
SET kartoteka_wynik = :kartoteka_wynik,
  kartoteka_odnosnik = :kartoteka_odnosnik,
  sprawdzono_kartoteke = :sprawdzono_kartoteke
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "kartoteka_wynik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.kartoteka_wynik),
                    ),
                    (
                        "kartoteka_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.kartoteka_odnosnik),
                    ),
                    (
                        "sprawdzono_kartoteke",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.sprawdzono_kartoteke,
                        ),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `przekaz_karte_czlowiekowi`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PrzekazKarteCzlowiekowiParams {
    pub id: i64,
}

/// Karta czeka na zgodę człowieka, np. bo reguły jej nie rozstrzygają.
pub fn przekaz_karte_czlowiekowi(
    room: &impl xmeta_room::MetaRoom,
    params: &PrzekazKarteCzlowiekowiParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kierownik_biura_przydzial_karty
SET stan = 'u_czlowieka'
WHERE id = :id"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `przydziel_karte`.
#[derive(Debug, Clone, PartialEq)]
pub struct PrzydzielKarteParams {
    pub dzial: KartyDzial,
    pub regula_id: Option<i64>,
    pub rozstrzygnieto: String,
    pub zatwierdzil: Option<String>,
    pub podpis: Option<String>,
    pub zatwierdzono: Option<String>,
    pub id: i64,
}

/// Karta przydzielona działowi według reguły albo za zgodą człowieka; kontakt trafił do działu,
/// więc znika z karty.
pub fn przydziel_karte(
    room: &impl xmeta_room::MetaRoom,
    params: &PrzydzielKarteParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kierownik_biura_przydzial_karty
SET stan = 'przydzielona',
  dzial = :dzial,
  regula_id = :regula_id,
  rozstrzygnieto = :rozstrzygnieto,
  zatwierdzil = :zatwierdzil,
  podpis = :podpis,
  zatwierdzono = :zatwierdzono,
  kontakt = NULL
WHERE id = :id"#,
                parameters: Vec::from([
                    ("dzial", xmeta_room::ToSqlValue::to_sql_value(&params.dzial)),
                    (
                        "regula_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.regula_id),
                    ),
                    (
                        "rozstrzygnieto",
                        xmeta_room::ToSqlValue::to_sql_value(&params.rozstrzygnieto),
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
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odrzuc_karte`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdrzucKarteParams {
    pub powod: String,
    pub rozstrzygnieto: String,
    pub zatwierdzil: Option<String>,
    pub podpis: Option<String>,
    pub zatwierdzono: Option<String>,
    pub id: i64,
}

/// Karta odrzucona z powodem, np. reklama; odrzucenie zgłoszenia klienta wymaga zgody człowieka.
pub fn odrzuc_karte(
    room: &impl xmeta_room::MetaRoom,
    params: &OdrzucKarteParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kierownik_biura_przydzial_karty
SET stan = 'odrzucona',
  powod = :powod,
  rozstrzygnieto = :rozstrzygnieto,
  zatwierdzil = :zatwierdzil,
  podpis = :podpis,
  zatwierdzono = :zatwierdzono,
  kontakt = NULL
WHERE id = :id"#,
                parameters: Vec::from([
                    ("powod", xmeta_room::ToSqlValue::to_sql_value(&params.powod)),
                    (
                        "rozstrzygnieto",
                        xmeta_room::ToSqlValue::to_sql_value(&params.rozstrzygnieto),
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
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// A row of `reguly_przydzialu`.
#[derive(Debug, Clone, PartialEq)]
pub struct RegulyPrzydzialuRow {
    pub id: i64,
    pub rodzaj_zgloszenia: KartyRodzajSprawy,
    pub dzial: KartyDzial,
    pub od: String,
    pub podpisal: String,
    pub podpis: String,
    pub zastapiona_od: Option<String>,
}

impl RegulyPrzydzialuRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(RegulyPrzydzialuRow {
            id: row.get("id")?,
            rodzaj_zgloszenia: row.get("rodzaj_zgloszenia")?,
            dzial: row.get("dzial")?,
            od: row.get("od")?,
            podpisal: row.get("podpisal")?,
            podpis: row.get("podpis")?,
            zastapiona_od: row.get("zastapiona_od")?,
        })
    }
}

/// Aktualne reguły przydziału według rodzaju zgłoszenia.
pub fn reguly_przydzialu(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<RegulyPrzydzialuRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM kierownik_biura_przydzial_reguly
WHERE zastapiona_od IS NULL
ORDER BY rodzaj_zgloszenia, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(RegulyPrzydzialuRow::from_row).collect()
}

/// A row of `historia_regul`.
#[derive(Debug, Clone, PartialEq)]
pub struct HistoriaRegulRow {
    pub id: i64,
    pub rodzaj_zgloszenia: KartyRodzajSprawy,
    pub dzial: KartyDzial,
    pub od: String,
    pub podpisal: String,
    pub podpis: String,
    pub zastapiona_od: Option<String>,
}

impl HistoriaRegulRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(HistoriaRegulRow {
            id: row.get("id")?,
            rodzaj_zgloszenia: row.get("rodzaj_zgloszenia")?,
            dzial: row.get("dzial")?,
            od: row.get("od")?,
            podpisal: row.get("podpisal")?,
            podpis: row.get("podpis")?,
            zastapiona_od: row.get("zastapiona_od")?,
        })
    }
}

/// Wszystkie reguły przydziału, także zastąpione, od najstarszej; z nich kierownik co tydzień
/// porównuje przydziały z regułami (dryf).
pub fn historia_regul(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<HistoriaRegulRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM kierownik_biura_przydzial_reguly
ORDER BY od, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(HistoriaRegulRow::from_row).collect()
}

/// The parameters of `zapisz_regule_przydzialu`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZapiszRegulePrzydzialuParams {
    pub rodzaj_zgloszenia: KartyRodzajSprawy,
    pub dzial: KartyDzial,
    pub od: String,
    pub podpisal: String,
    pub podpis: String,
}

/// Nowa reguła podpisana przez właściciela biura; poprzednią dla tego rodzaju trzeba najpierw
/// zastąpić.
pub fn zapisz_regule_przydzialu(
    room: &impl xmeta_room::MetaRoom,
    params: &ZapiszRegulePrzydzialuParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO kierownik_biura_przydzial_reguly (rodzaj_zgloszenia, dzial, od, podpisal, podpis)
VALUES (:rodzaj_zgloszenia, :dzial, :od, :podpisal, :podpis)"#,
                parameters: Vec::from([
                    (
                        "rodzaj_zgloszenia",
                        xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj_zgloszenia),
                    ),
                    ("dzial", xmeta_room::ToSqlValue::to_sql_value(&params.dzial)),
                    ("od", xmeta_room::ToSqlValue::to_sql_value(&params.od)),
                    ("podpisal", xmeta_room::ToSqlValue::to_sql_value(&params.podpisal)),
                    ("podpis", xmeta_room::ToSqlValue::to_sql_value(&params.podpis)),
                ]),
            },
        ],
    )
}

/// The parameters of `zastap_regule_przydzialu`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZastapRegulePrzydzialuParams {
    pub zastapiona_od: String,
    pub id: i64,
}

/// Reguła przestaje obowiązywać, bo właściciel biura podpisał nową.
pub fn zastap_regule_przydzialu(
    room: &impl xmeta_room::MetaRoom,
    params: &ZastapRegulePrzydzialuParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE kierownik_biura_przydzial_reguly
SET zastapiona_od = :zastapiona_od
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "zastapiona_od",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zastapiona_od),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}
