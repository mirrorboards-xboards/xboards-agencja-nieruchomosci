//! The kind `oferta-oferty` of the room `cases/oferta`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind: its name, the kinds it requires and its migrations in the order of their
/// numbers.
pub const KIND: xmeta_room::Kind = xmeta_room::Kind {
    name: "oferta-oferty",
    requires: &[
        &super::oferta_zainteresowani::KIND,
        &super::sprawa_decyzje::KIND,
        &super::sprawa_dokumenty::KIND,
        &super::sprawa_korespondencja::KIND,
        &super::sprawa_metryka::KIND,
        &super::sprawa_uczestnicy::KIND,
        &super::sprawa_umowy::KIND,
        &super::sprawa_zgody::KIND,
    ],
    migrations: &[
        r#"-- Oferty zainteresowanych i kontroferty klienta jako łańcuch wpisów (art. 66 i 68 KC). Pisze
-- Opiekun oferty. Żadnego wpisu się nie zmienia; odpowiedź ze zmianą warunków jest nowym wpisem,
-- który wskazuje ofertę, na którą odpowiada.
--
-- Oferta zainteresowanego przychodzi wpływem albo notatką pośrednika z rozmowy. Opiekun
-- przedstawia ją klientowi (przedstawiono), a klient ją przyjmuje, odrzuca albo składa kontrofertę
-- (decyzja_klienta) zgodą odpowiedz_na_oferte, która dotyczy tej oferty. Kontroferta klienta to
-- wpis od_klienta, który wskazuje ofertę zainteresowanego i tę samą zgodę. Odpowiedź
-- zainteresowanego na kontrofertę (odpowiedz_drugiej_strony) przychodzi wpływem; jego nowa oferta
-- jest kolejnym wpisem.
--
-- Odpowiedź biura na ofertę zainteresowanego to wysyłka, która zawsze czeka na zgodę człowieka
-- (D15), bo przyjęcie może wiązać klienta; przyjęcie nie wychodzi, dopóki stoi znacznik
-- wątpliwości. Oferty klienta biura bez zgody umowy na pośrednictwo dla obu stron Opiekun nie
-- przedstawia klientowi bez zatwierdzonej decyzji „inna”, która wskazuje tę ofertę (R1).
--
-- Teczka oferty nie przyjmuje pieniędzy: zadatek i kaucja w ofercie to tylko warunki.
-- Finansowanie jest twierdzeniem składającego; biuro go nie sprawdza.
CREATE TABLE oferta_oferty_wpisy (
  id TEXT PRIMARY KEY,
  dotyczy TEXT NOT NULL CHECK (dotyczy IN ('sprzedaz', 'najem')),
  kierunek TEXT NOT NULL CHECK (kierunek IN ('od_zainteresowanego', 'od_klienta')),
  -- Zainteresowany, który składa ofertę albo do którego idzie kontroferta klienta.
  osoba_id TEXT NOT NULL REFERENCES oferta_zainteresowani_osoby (id),
  odpowiada_na_id TEXT REFERENCES oferta_oferty_wpisy (id),
  -- Skąd teczka ma ofertę zainteresowanego: wpływ albo notatka pośrednika z rozmowy.
  wplyw_id TEXT REFERENCES sprawa_korespondencja_wplywy (id),
  dokument_id TEXT REFERENCES sprawa_dokumenty_wpisy (id),
  wpisano TEXT NOT NULL,

  -- Warunki: cena albo czynsz, zadatek albo kaucja, terminy umowy i wydania nieruchomości.
  kwota INTEGER NOT NULL CHECK (kwota > 0),
  zadatek_albo_kaucja INTEGER CHECK (zadatek_albo_kaucja >= 0),
  termin_umowy TEXT,
  termin_wydania TEXT,
  finansowanie TEXT,
  warunki_zastrzezone TEXT,

  -- Związanie ofertą (art. 66 § 2 KC): termin z oferty albo sposób złożenia, z którego termin
  -- liczy program.
  wiaze_do TEXT,
  sposob_zlozenia TEXT CHECK (sposob_zlozenia IN ('w_rozmowie', 'na_pismie')),

  -- Przebieg oferty zainteresowanego: przedstawienie klientowi, jego decyzja ze zgodą, wysyłka
  -- odpowiedzi i decyzja człowieka o ofercie klienta biura bez zgody na obie strony.
  przedstawiono TEXT,
  decyzja_klienta TEXT CHECK (decyzja_klienta IN ('przyjeta', 'odrzucona', 'kontroferta')),
  zgoda_id TEXT REFERENCES sprawa_zgody_wpisy (id),
  odpowiedz_wysylka_id TEXT REFERENCES sprawa_korespondencja_wysylki (id),
  obie_strony_decyzja_id TEXT REFERENCES sprawa_decyzje_wpisy (id),

  -- Odpowiedź zainteresowanego na kontrofertę klienta, z wpływem.
  odpowiedz_drugiej_strony TEXT CHECK (odpowiedz_drugiej_strony IN ('przyjeta', 'odrzucona', 'kontroferta')),
  odpowiedz_wplyw_id TEXT REFERENCES sprawa_korespondencja_wplywy (id),

  CONSTRAINT oferta_ma_termin_albo_sposob_zlozenia CHECK (wiaze_do IS NOT NULL OR sposob_zlozenia IS NOT NULL),
  CONSTRAINT oferta_zainteresowanego_z_wplywu_albo_notatki CHECK (
    (kierunek = 'od_zainteresowanego' AND (wplyw_id IS NOT NULL) + (dokument_id IS NOT NULL) = 1)
    OR (kierunek = 'od_klienta' AND wplyw_id IS NULL AND dokument_id IS NULL)
  ),
  CONSTRAINT kontroferta_wskazuje_oferte_i_zgode CHECK (
    kierunek <> 'od_klienta' OR (odpowiada_na_id IS NOT NULL AND zgoda_id IS NOT NULL)
  ),
  CONSTRAINT przebieg_tylko_oferty_zainteresowanego CHECK (
    kierunek = 'od_zainteresowanego'
    OR (przedstawiono IS NULL AND decyzja_klienta IS NULL AND odpowiedz_wysylka_id IS NULL AND obie_strony_decyzja_id IS NULL)
  ),
  CONSTRAINT decyzja_klienta_ze_zgoda CHECK (kierunek <> 'od_zainteresowanego' OR (decyzja_klienta IS NULL) = (zgoda_id IS NULL)),
  CONSTRAINT decyzja_po_przedstawieniu CHECK (decyzja_klienta IS NULL OR przedstawiono IS NOT NULL),
  CONSTRAINT odpowiedz_po_decyzji CHECK (odpowiedz_wysylka_id IS NULL OR decyzja_klienta IS NOT NULL),
  CONSTRAINT odpowiedz_drugiej_strony_na_kontroferte CHECK (
    (kierunek = 'od_klienta' OR odpowiedz_drugiej_strony IS NULL)
    AND (odpowiedz_drugiej_strony IS NULL) = (odpowiedz_wplyw_id IS NULL)
  )
) STRICT;

-- Przebieg sprawdza zmiany, więc oferta zainteresowanego powstaje nieprzedstawiona, a kontroferta
-- klienta bez odpowiedzi zainteresowanego.
CREATE TRIGGER oferta_oferty_powstaje_bez_przebiegu
BEFORE INSERT ON oferta_oferty_wpisy
WHEN NEW.przedstawiono IS NOT NULL
  OR NEW.decyzja_klienta IS NOT NULL
  OR NEW.odpowiedz_wysylka_id IS NOT NULL
  OR NEW.obie_strony_decyzja_id IS NOT NULL
  OR NEW.odpowiedz_drugiej_strony IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'oferta powstaje bez przedstawienia, decyzji i odpowiedzi');
END;

-- Kontroferta klienta odpowiada na ofertę tego samego zainteresowanego, o której klient
-- zdecydował „kontroferta”, i wskazuje zgodę z tej decyzji.
CREATE TRIGGER oferta_oferty_kontroferta_klienta_wskazuje_decyzje
BEFORE INSERT ON oferta_oferty_wpisy
WHEN NEW.kierunek = 'od_klienta'
  AND NOT EXISTS (
    SELECT 1
    FROM oferta_oferty_wpisy AS poprzednia
    WHERE poprzednia.id = NEW.odpowiada_na_id
      AND poprzednia.kierunek = 'od_zainteresowanego'
      AND poprzednia.decyzja_klienta = 'kontroferta'
      AND poprzednia.zgoda_id = NEW.zgoda_id
      AND poprzednia.osoba_id = NEW.osoba_id
      AND poprzednia.dotyczy = NEW.dotyczy
  )
BEGIN
  SELECT RAISE(ABORT, 'kontroferta klienta odpowiada na ofertę zainteresowanego z decyzją kontroferta i wskazuje zgodę z tej decyzji');
END;

-- Nowa oferta zainteresowanego w łańcuchu odpowiada na kontrofertę klienta, na którą ten sam
-- zainteresowany odpowiedział kontrofertą.
CREATE TRIGGER oferta_oferty_kontroferta_zainteresowanego_wskazuje_odpowiedz
BEFORE INSERT ON oferta_oferty_wpisy
WHEN NEW.kierunek = 'od_zainteresowanego'
  AND NEW.odpowiada_na_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM oferta_oferty_wpisy AS poprzednia
    WHERE poprzednia.id = NEW.odpowiada_na_id
      AND poprzednia.kierunek = 'od_klienta'
      AND poprzednia.odpowiedz_drugiej_strony = 'kontroferta'
      AND poprzednia.osoba_id = NEW.osoba_id
      AND poprzednia.dotyczy = NEW.dotyczy
  )
BEGIN
  SELECT RAISE(ABORT, 'kontroferta zainteresowanego odpowiada na kontrofertę klienta, na którą ten zainteresowany odpowiedział kontrofertą');
END;

-- Decyzję o ofercie podejmuje klient: zgoda odpowiedz_na_oferte uczestnika z obowiązującą rolą
-- klient, która dotyczy tej oferty.
CREATE TRIGGER oferta_oferty_decyzja_ze_zgoda_klienta
BEFORE UPDATE OF decyzja_klienta, zgoda_id ON oferta_oferty_wpisy
WHEN NEW.decyzja_klienta IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_zgody_wpisy AS zgoda
    JOIN sprawa_uczestnicy_role AS rola ON rola.uczestnik_id = zgoda.uczestnik_id
    WHERE zgoda.id = NEW.zgoda_id
      AND zgoda.rodzaj = 'odpowiedz_na_oferte'
      AND zgoda.dotyczy_tabela = 'oferta_oferty_wpisy'
      AND zgoda.dotyczy_id = NEW.id
      AND rola.rola = 'klient'
      AND rola.zakonczono IS NULL
  )
BEGIN
  SELECT RAISE(ABORT, 'decyzja klienta o ofercie wskazuje jego zgodę odpowiedz_na_oferte, która dotyczy tej oferty');
END;

-- R1: ofertę klienta tego biura Opiekun przedstawia bez decyzji człowieka tylko wtedy, gdy umowa
-- pośrednictwa pozwala biuru działać dla obu stron.
CREATE TRIGGER oferta_oferty_klient_biura_bez_obu_stron_za_zgoda
BEFORE UPDATE OF przedstawiono ON oferta_oferty_wpisy
WHEN NEW.przedstawiono IS NOT NULL
  AND EXISTS (
    SELECT 1
    FROM oferta_zainteresowani_osoby
    WHERE id = NEW.osoba_id
      AND dziala_jako = 'klient_biura'
  )
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_umowy_wpisy
    WHERE rodzaj = 'posrednictwo'
      AND stan = 'zawarta'
      AND obie_strony = 1
  )
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_decyzje_zatwierdzone AS decyzja
    WHERE decyzja.id = NEW.obie_strony_decyzja_id
      AND decyzja.rodzaj = 'inna'
      AND decyzja.dotyczy_tabela = 'oferta_oferty_wpisy'
      AND decyzja.dotyczy_id = NEW.id
  )
BEGIN
  SELECT RAISE(ABORT, 'ofertę klienta biura bez zgody umowy na obie strony przedstawia się klientowi tylko z zatwierdzoną decyzją inna, która ją wskazuje');
END;

-- Odpowiedź wskazuje wysyłkę, która jeszcze nie wyszła, żeby jej wysłanie przeszło przez zgodę.
CREATE TRIGGER oferta_oferty_odpowiedz_przed_wyslaniem
BEFORE UPDATE OF odpowiedz_wysylka_id ON oferta_oferty_wpisy
WHEN NEW.odpowiedz_wysylka_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_korespondencja_wysylki
    WHERE id = NEW.odpowiedz_wysylka_id
      AND stan IN ('projekt', 'czeka_na_zgode')
  )
BEGIN
  SELECT RAISE(ABORT, 'odpowiedź na ofertę wskazuje wysyłkę, która jeszcze nie wyszła');
END;

-- Warunków oferty się nie zmienia; przedstawienie, decyzję klienta, odpowiedź i odpowiedź
-- zainteresowanego wpisuje się raz.
CREATE TRIGGER oferta_oferty_wpisy_bez_zmian
BEFORE UPDATE ON oferta_oferty_wpisy
WHEN NEW.id IS NOT OLD.id
  OR NEW.dotyczy IS NOT OLD.dotyczy
  OR NEW.kierunek IS NOT OLD.kierunek
  OR NEW.osoba_id IS NOT OLD.osoba_id
  OR NEW.odpowiada_na_id IS NOT OLD.odpowiada_na_id
  OR NEW.wplyw_id IS NOT OLD.wplyw_id
  OR NEW.dokument_id IS NOT OLD.dokument_id
  OR NEW.wpisano IS NOT OLD.wpisano
  OR NEW.kwota IS NOT OLD.kwota
  OR NEW.zadatek_albo_kaucja IS NOT OLD.zadatek_albo_kaucja
  OR NEW.termin_umowy IS NOT OLD.termin_umowy
  OR NEW.termin_wydania IS NOT OLD.termin_wydania
  OR NEW.finansowanie IS NOT OLD.finansowanie
  OR NEW.warunki_zastrzezone IS NOT OLD.warunki_zastrzezone
  OR NEW.wiaze_do IS NOT OLD.wiaze_do
  OR NEW.sposob_zlozenia IS NOT OLD.sposob_zlozenia
  OR (OLD.przedstawiono IS NOT NULL AND NEW.przedstawiono IS NOT OLD.przedstawiono)
  OR (OLD.decyzja_klienta IS NOT NULL AND NEW.decyzja_klienta IS NOT OLD.decyzja_klienta)
  OR (OLD.zgoda_id IS NOT NULL AND NEW.zgoda_id IS NOT OLD.zgoda_id)
  OR (OLD.odpowiedz_wysylka_id IS NOT NULL AND NEW.odpowiedz_wysylka_id IS NOT OLD.odpowiedz_wysylka_id)
  OR (OLD.obie_strony_decyzja_id IS NOT NULL AND NEW.obie_strony_decyzja_id IS NOT OLD.obie_strony_decyzja_id)
  OR (OLD.odpowiedz_drugiej_strony IS NOT NULL AND NEW.odpowiedz_drugiej_strony IS NOT OLD.odpowiedz_drugiej_strony)
  OR (OLD.odpowiedz_wplyw_id IS NOT NULL AND NEW.odpowiedz_wplyw_id IS NOT OLD.odpowiedz_wplyw_id)
BEGIN
  SELECT RAISE(ABORT, 'oferty się nie zmienia; przedstawienie, decyzję klienta i odpowiedzi wpisuje się raz');
END;

-- Odpowiedź na ofertę zawsze czeka na zgodę człowieka, także gdy idzie według wzoru biura.
CREATE TRIGGER oferta_oferty_odpowiedz_czeka_na_zgode
BEFORE UPDATE OF stan ON sprawa_korespondencja_wysylki
WHEN NEW.stan = 'wyslana'
  AND OLD.stan = 'projekt'
  AND EXISTS (
    SELECT 1
    FROM oferta_oferty_wpisy
    WHERE odpowiedz_wysylka_id = NEW.id
  )
BEGIN
  SELECT RAISE(ABORT, 'odpowiedź na ofertę przed wysłaniem czeka na zgodę');
END;

-- Przyjęcie oferty nie wychodzi, dopóki stoi znacznik wątpliwości Kontrolera.
CREATE TRIGGER oferta_oferty_watpliwosc_blokuje_przyjecie
BEFORE UPDATE OF stan ON sprawa_korespondencja_wysylki
WHEN NEW.stan = 'wyslana'
  AND EXISTS (
    SELECT 1
    FROM oferta_oferty_wpisy
    WHERE odpowiedz_wysylka_id = NEW.id
      AND decyzja_klienta = 'przyjeta'
  )
  AND EXISTS (
    SELECT 1
    FROM sprawa_metryka_watpliwosci
    WHERE zdjecie_decyzja_id IS NULL
  )
BEGIN
  SELECT RAISE(ABORT, 'dopóki stoi znacznik wątpliwości, przyjęcie oferty nie wychodzi');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER oferta_oferty_wpisy_bez_zastepowania
BEFORE INSERT ON oferta_oferty_wpisy
WHEN EXISTS (
  SELECT 1
  FROM oferta_oferty_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER oferta_oferty_wpisy_bez_usuwania
BEFORE DELETE ON oferta_oferty_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;
"#,
    ],
};

/// The values of column `wpisy.dotyczy`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyDotyczy {
    Sprzedaz,
    Najem,
}

impl WpisyDotyczy {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyDotyczy] = &[WpisyDotyczy::Sprzedaz, WpisyDotyczy::Najem];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyDotyczy::Sprzedaz => "sprzedaz",
            WpisyDotyczy::Najem => "najem",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyDotyczy {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "sprzedaz" => Ok(WpisyDotyczy::Sprzedaz),
                    "najem" => Ok(WpisyDotyczy::Najem),
                    other => Err(format!("{other} is no value of WpisyDotyczy")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyDotyczy {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wpisy.kierunek`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyKierunek {
    OdZainteresowanego,
    OdKlienta,
}

impl WpisyKierunek {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyKierunek] = &[
        WpisyKierunek::OdZainteresowanego,
        WpisyKierunek::OdKlienta,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyKierunek::OdZainteresowanego => "od_zainteresowanego",
            WpisyKierunek::OdKlienta => "od_klienta",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyKierunek {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "od_zainteresowanego" => Ok(WpisyKierunek::OdZainteresowanego),
                    "od_klienta" => Ok(WpisyKierunek::OdKlienta),
                    other => Err(format!("{other} is no value of WpisyKierunek")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyKierunek {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wpisy.sposob_zlozenia`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisySposobZlozenia {
    WRozmowie,
    NaPismie,
}

impl WpisySposobZlozenia {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisySposobZlozenia] = &[
        WpisySposobZlozenia::WRozmowie,
        WpisySposobZlozenia::NaPismie,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisySposobZlozenia::WRozmowie => "w_rozmowie",
            WpisySposobZlozenia::NaPismie => "na_pismie",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisySposobZlozenia {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "w_rozmowie" => Ok(WpisySposobZlozenia::WRozmowie),
                    "na_pismie" => Ok(WpisySposobZlozenia::NaPismie),
                    other => Err(format!("{other} is no value of WpisySposobZlozenia")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisySposobZlozenia {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wpisy.decyzja_klienta`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyDecyzjaKlienta {
    Przyjeta,
    Odrzucona,
    Kontroferta,
}

impl WpisyDecyzjaKlienta {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyDecyzjaKlienta] = &[
        WpisyDecyzjaKlienta::Przyjeta,
        WpisyDecyzjaKlienta::Odrzucona,
        WpisyDecyzjaKlienta::Kontroferta,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyDecyzjaKlienta::Przyjeta => "przyjeta",
            WpisyDecyzjaKlienta::Odrzucona => "odrzucona",
            WpisyDecyzjaKlienta::Kontroferta => "kontroferta",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyDecyzjaKlienta {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "przyjeta" => Ok(WpisyDecyzjaKlienta::Przyjeta),
                    "odrzucona" => Ok(WpisyDecyzjaKlienta::Odrzucona),
                    "kontroferta" => Ok(WpisyDecyzjaKlienta::Kontroferta),
                    other => Err(format!("{other} is no value of WpisyDecyzjaKlienta")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyDecyzjaKlienta {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// A row of `oferty`.
#[derive(Debug, Clone, PartialEq)]
pub struct OfertyRow {
    pub id: String,
    pub dotyczy: WpisyDotyczy,
    pub kierunek: WpisyKierunek,
    pub osoba_id: String,
    pub odpowiada_na_id: Option<String>,
    pub wplyw_id: Option<String>,
    pub dokument_id: Option<String>,
    pub wpisano: String,
    pub kwota: i64,
    pub zadatek_albo_kaucja: Option<i64>,
    pub termin_umowy: Option<String>,
    pub termin_wydania: Option<String>,
    pub finansowanie: Option<String>,
    pub warunki_zastrzezone: Option<String>,
    pub wiaze_do: Option<String>,
    pub sposob_zlozenia: Option<WpisySposobZlozenia>,
    pub przedstawiono: Option<String>,
    pub decyzja_klienta: Option<WpisyDecyzjaKlienta>,
    pub zgoda_id: Option<String>,
    pub odpowiedz_wysylka_id: Option<String>,
    pub obie_strony_decyzja_id: Option<String>,
    pub odpowiedz_drugiej_strony: Option<WpisyDecyzjaKlienta>,
    pub odpowiedz_wplyw_id: Option<String>,
}

impl OfertyRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(OfertyRow {
            id: row.get("id")?,
            dotyczy: row.get("dotyczy")?,
            kierunek: row.get("kierunek")?,
            osoba_id: row.get("osoba_id")?,
            odpowiada_na_id: row.get("odpowiada_na_id")?,
            wplyw_id: row.get("wplyw_id")?,
            dokument_id: row.get("dokument_id")?,
            wpisano: row.get("wpisano")?,
            kwota: row.get("kwota")?,
            zadatek_albo_kaucja: row.get("zadatek_albo_kaucja")?,
            termin_umowy: row.get("termin_umowy")?,
            termin_wydania: row.get("termin_wydania")?,
            finansowanie: row.get("finansowanie")?,
            warunki_zastrzezone: row.get("warunki_zastrzezone")?,
            wiaze_do: row.get("wiaze_do")?,
            sposob_zlozenia: row.get("sposob_zlozenia")?,
            przedstawiono: row.get("przedstawiono")?,
            decyzja_klienta: row.get("decyzja_klienta")?,
            zgoda_id: row.get("zgoda_id")?,
            odpowiedz_wysylka_id: row.get("odpowiedz_wysylka_id")?,
            obie_strony_decyzja_id: row.get("obie_strony_decyzja_id")?,
            odpowiedz_drugiej_strony: row.get("odpowiedz_drugiej_strony")?,
            odpowiedz_wplyw_id: row.get("odpowiedz_wplyw_id")?,
        })
    }
}

/// Oferty zainteresowanych i kontroferty klienta w kolejności wpisania.
pub fn oferty(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<OfertyRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM oferta_oferty_wpisy
ORDER BY wpisano, rowid"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(OfertyRow::from_row).collect()
}

/// A row of `oferty_do_przedstawienia`.
#[derive(Debug, Clone, PartialEq)]
pub struct OfertyDoPrzedstawieniaRow {
    pub id: String,
    pub dotyczy: WpisyDotyczy,
    pub kierunek: WpisyKierunek,
    pub osoba_id: String,
    pub odpowiada_na_id: Option<String>,
    pub wplyw_id: Option<String>,
    pub dokument_id: Option<String>,
    pub wpisano: String,
    pub kwota: i64,
    pub zadatek_albo_kaucja: Option<i64>,
    pub termin_umowy: Option<String>,
    pub termin_wydania: Option<String>,
    pub finansowanie: Option<String>,
    pub warunki_zastrzezone: Option<String>,
    pub wiaze_do: Option<String>,
    pub sposob_zlozenia: Option<WpisySposobZlozenia>,
    pub przedstawiono: Option<String>,
    pub decyzja_klienta: Option<WpisyDecyzjaKlienta>,
    pub zgoda_id: Option<String>,
    pub odpowiedz_wysylka_id: Option<String>,
    pub obie_strony_decyzja_id: Option<String>,
    pub odpowiedz_drugiej_strony: Option<WpisyDecyzjaKlienta>,
    pub odpowiedz_wplyw_id: Option<String>,
}

impl OfertyDoPrzedstawieniaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(OfertyDoPrzedstawieniaRow {
            id: row.get("id")?,
            dotyczy: row.get("dotyczy")?,
            kierunek: row.get("kierunek")?,
            osoba_id: row.get("osoba_id")?,
            odpowiada_na_id: row.get("odpowiada_na_id")?,
            wplyw_id: row.get("wplyw_id")?,
            dokument_id: row.get("dokument_id")?,
            wpisano: row.get("wpisano")?,
            kwota: row.get("kwota")?,
            zadatek_albo_kaucja: row.get("zadatek_albo_kaucja")?,
            termin_umowy: row.get("termin_umowy")?,
            termin_wydania: row.get("termin_wydania")?,
            finansowanie: row.get("finansowanie")?,
            warunki_zastrzezone: row.get("warunki_zastrzezone")?,
            wiaze_do: row.get("wiaze_do")?,
            sposob_zlozenia: row.get("sposob_zlozenia")?,
            przedstawiono: row.get("przedstawiono")?,
            decyzja_klienta: row.get("decyzja_klienta")?,
            zgoda_id: row.get("zgoda_id")?,
            odpowiedz_wysylka_id: row.get("odpowiedz_wysylka_id")?,
            obie_strony_decyzja_id: row.get("obie_strony_decyzja_id")?,
            odpowiedz_drugiej_strony: row.get("odpowiedz_drugiej_strony")?,
            odpowiedz_wplyw_id: row.get("odpowiedz_wplyw_id")?,
        })
    }
}

/// Oferty zainteresowanych, których Opiekun jeszcze nie przedstawił klientowi.
pub fn oferty_do_przedstawienia(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<OfertyDoPrzedstawieniaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM oferta_oferty_wpisy
WHERE kierunek = 'od_zainteresowanego'
  AND przedstawiono IS NULL
ORDER BY wpisano, rowid"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(OfertyDoPrzedstawieniaRow::from_row).collect()
}

/// The parameters of `wpisz_oferte_zainteresowanego`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszOferteZainteresowanegoParams {
    pub id: String,
    pub dotyczy: WpisyDotyczy,
    pub osoba_id: String,
    pub odpowiada_na_id: Option<String>,
    pub wplyw_id: Option<String>,
    pub dokument_id: Option<String>,
    pub wpisano: String,
    pub kwota: i64,
    pub zadatek_albo_kaucja: Option<i64>,
    pub termin_umowy: Option<String>,
    pub termin_wydania: Option<String>,
    pub finansowanie: Option<String>,
    pub warunki_zastrzezone: Option<String>,
    pub wiaze_do: Option<String>,
    pub sposob_zlozenia: Option<WpisySposobZlozenia>,
}

/// Oferta zainteresowanego z wpływem albo notatką pośrednika, z warunkami i terminem związania
/// albo sposobem złożenia. Nowa oferta w łańcuchu wskazuje kontrofertę klienta, na którą
/// odpowiada. Identyfikator (UUID) nadaje aplikacja, bo wskazują go zgoda, decyzja i katalog ofert.
pub fn wpisz_oferte_zainteresowanego(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszOferteZainteresowanegoParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO oferta_oferty_wpisy (id, dotyczy, kierunek, osoba_id, odpowiada_na_id, wplyw_id,
  dokument_id, wpisano, kwota, zadatek_albo_kaucja, termin_umowy, termin_wydania, finansowanie,
  warunki_zastrzezone, wiaze_do, sposob_zlozenia)
VALUES (:id, :dotyczy, 'od_zainteresowanego', :osoba_id, :odpowiada_na_id, :wplyw_id, :dokument_id,
  :wpisano, :kwota, :zadatek_albo_kaucja, :termin_umowy, :termin_wydania, :finansowanie,
  :warunki_zastrzezone, :wiaze_do, :sposob_zlozenia)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("dotyczy", xmeta_room::ToSqlValue::to_sql_value(&params.dotyczy)),
                    ("osoba_id", xmeta_room::ToSqlValue::to_sql_value(&params.osoba_id)),
                    (
                        "odpowiada_na_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.odpowiada_na_id),
                    ),
                    ("wplyw_id", xmeta_room::ToSqlValue::to_sql_value(&params.wplyw_id)),
                    (
                        "dokument_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dokument_id),
                    ),
                    ("wpisano", xmeta_room::ToSqlValue::to_sql_value(&params.wpisano)),
                    ("kwota", xmeta_room::ToSqlValue::to_sql_value(&params.kwota)),
                    (
                        "zadatek_albo_kaucja",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zadatek_albo_kaucja),
                    ),
                    (
                        "termin_umowy",
                        xmeta_room::ToSqlValue::to_sql_value(&params.termin_umowy),
                    ),
                    (
                        "termin_wydania",
                        xmeta_room::ToSqlValue::to_sql_value(&params.termin_wydania),
                    ),
                    (
                        "finansowanie",
                        xmeta_room::ToSqlValue::to_sql_value(&params.finansowanie),
                    ),
                    (
                        "warunki_zastrzezone",
                        xmeta_room::ToSqlValue::to_sql_value(&params.warunki_zastrzezone),
                    ),
                    ("wiaze_do", xmeta_room::ToSqlValue::to_sql_value(&params.wiaze_do)),
                    (
                        "sposob_zlozenia",
                        xmeta_room::ToSqlValue::to_sql_value(&params.sposob_zlozenia),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_przedstawienie_oferty`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujPrzedstawienieOfertyParams {
    pub przedstawiono: String,
    pub obie_strony_decyzja_id: Option<String>,
    pub id: String,
}

/// Chwila, w której klient dostał ofertę; oferta klienta biura bez zgody umowy na obie strony
/// wskazuje zatwierdzoną decyzję człowieka.
pub fn odnotuj_przedstawienie_oferty(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujPrzedstawienieOfertyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE oferta_oferty_wpisy
SET przedstawiono = :przedstawiono,
  obie_strony_decyzja_id = :obie_strony_decyzja_id
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "przedstawiono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.przedstawiono),
                    ),
                    (
                        "obie_strony_decyzja_id",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.obie_strony_decyzja_id,
                        ),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_decyzje_klienta_o_ofercie`.
#[derive(Debug, Clone, PartialEq)]
pub struct OdnotujDecyzjeKlientaOOfercieParams {
    pub decyzja_klienta: WpisyDecyzjaKlienta,
    pub zgoda_id: String,
    pub id: String,
}

/// Przyjęcie, odrzucenie albo kontroferta klienta z jego zgodą odpowiedz_na_oferte.
pub fn odnotuj_decyzje_klienta_o_ofercie(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujDecyzjeKlientaOOfercieParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE oferta_oferty_wpisy
SET decyzja_klienta = :decyzja_klienta,
  zgoda_id = :zgoda_id
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "decyzja_klienta",
                        xmeta_room::ToSqlValue::to_sql_value(&params.decyzja_klienta),
                    ),
                    ("zgoda_id", xmeta_room::ToSqlValue::to_sql_value(&params.zgoda_id)),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `wpisz_kontroferte_klienta`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszKontroferteKlientaParams {
    pub id: String,
    pub dotyczy: WpisyDotyczy,
    pub osoba_id: String,
    pub odpowiada_na_id: String,
    pub wpisano: String,
    pub kwota: i64,
    pub zadatek_albo_kaucja: Option<i64>,
    pub termin_umowy: Option<String>,
    pub termin_wydania: Option<String>,
    pub warunki_zastrzezone: Option<String>,
    pub wiaze_do: Option<String>,
    pub sposob_zlozenia: Option<WpisySposobZlozenia>,
    pub zgoda_id: String,
}

/// Kontroferta klienta jako nowy wpis, który wskazuje ofertę zainteresowanego i zgodę z decyzji
/// klienta.
pub fn wpisz_kontroferte_klienta(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszKontroferteKlientaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO oferta_oferty_wpisy (id, dotyczy, kierunek, osoba_id, odpowiada_na_id, wpisano, kwota,
  zadatek_albo_kaucja, termin_umowy, termin_wydania, warunki_zastrzezone, wiaze_do,
  sposob_zlozenia, zgoda_id)
VALUES (:id, :dotyczy, 'od_klienta', :osoba_id, :odpowiada_na_id, :wpisano, :kwota,
  :zadatek_albo_kaucja, :termin_umowy, :termin_wydania, :warunki_zastrzezone, :wiaze_do,
  :sposob_zlozenia, :zgoda_id)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("dotyczy", xmeta_room::ToSqlValue::to_sql_value(&params.dotyczy)),
                    ("osoba_id", xmeta_room::ToSqlValue::to_sql_value(&params.osoba_id)),
                    (
                        "odpowiada_na_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.odpowiada_na_id),
                    ),
                    ("wpisano", xmeta_room::ToSqlValue::to_sql_value(&params.wpisano)),
                    ("kwota", xmeta_room::ToSqlValue::to_sql_value(&params.kwota)),
                    (
                        "zadatek_albo_kaucja",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zadatek_albo_kaucja),
                    ),
                    (
                        "termin_umowy",
                        xmeta_room::ToSqlValue::to_sql_value(&params.termin_umowy),
                    ),
                    (
                        "termin_wydania",
                        xmeta_room::ToSqlValue::to_sql_value(&params.termin_wydania),
                    ),
                    (
                        "warunki_zastrzezone",
                        xmeta_room::ToSqlValue::to_sql_value(&params.warunki_zastrzezone),
                    ),
                    ("wiaze_do", xmeta_room::ToSqlValue::to_sql_value(&params.wiaze_do)),
                    (
                        "sposob_zlozenia",
                        xmeta_room::ToSqlValue::to_sql_value(&params.sposob_zlozenia),
                    ),
                    ("zgoda_id", xmeta_room::ToSqlValue::to_sql_value(&params.zgoda_id)),
                ]),
            },
        ],
    )
}

/// The parameters of `wskaz_odpowiedz_na_oferte`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WskazOdpowiedzNaOferteParams {
    pub odpowiedz_wysylka_id: String,
    pub id: String,
}

/// Wysyłka z odpowiedzią na ofertę zainteresowanego; jeszcze niewysłana, bo wychodzi tylko za
/// zgodą człowieka.
pub fn wskaz_odpowiedz_na_oferte(
    room: &impl xmeta_room::MetaRoom,
    params: &WskazOdpowiedzNaOferteParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE oferta_oferty_wpisy
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

/// The parameters of `odnotuj_odpowiedz_zainteresowanego`.
#[derive(Debug, Clone, PartialEq)]
pub struct OdnotujOdpowiedzZainteresowanegoParams {
    pub odpowiedz_drugiej_strony: WpisyDecyzjaKlienta,
    pub odpowiedz_wplyw_id: String,
    pub id: String,
}

/// Odpowiedź zainteresowanego na kontrofertę klienta, z wpływem, w którym przyszła.
pub fn odnotuj_odpowiedz_zainteresowanego(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujOdpowiedzZainteresowanegoParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE oferta_oferty_wpisy
SET odpowiedz_drugiej_strony = :odpowiedz_drugiej_strony,
  odpowiedz_wplyw_id = :odpowiedz_wplyw_id
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "odpowiedz_drugiej_strony",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.odpowiedz_drugiej_strony,
                        ),
                    ),
                    (
                        "odpowiedz_wplyw_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.odpowiedz_wplyw_id),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}
