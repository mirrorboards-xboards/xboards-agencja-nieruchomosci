//! The kind `poszukiwanie-oferty` of the room `cases/poszukiwanie`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind's name.
pub const KIND: &str = "poszukiwanie-oferty";

/// The migrations of the kind, in the order of their numbers.
pub const MIGRATIONS: &[&str] = &[
    r#"-- Oferty klienta i odpowiedzi drugiej strony jako łańcuch wpisów (art. 66 i 68 KC), na wzór
-- oferta-oferty ze stroną odwrotną. Pisze Opiekun poszukującego. Kwoty w groszach. Żadnego wpisu
-- się nie zmienia; odpowiedź ze zmianą warunków jest nowym wpisem, który wskazuje ofertę, na którą
-- odpowiada.
--
-- Oferta klienta (od_klienta) dotyczy propozycji, którą klient dostał. Opiekun przygotowuje ją
-- z klientem, klient potwierdza jej treść zgodą potwierdzenie_oferty, która dotyczy tej oferty,
-- a wysyłka czeka na zgodę człowieka (D15). Druga strona odpowiada wpływem: przyjmuje, odrzuca
-- albo składa kontrofertę (odpowiedz_drugiej_strony), a kontroferta jest wpisem
-- od_drugiej_strony. Klient przyjmuje ją albo odrzuca zgodą potwierdzenie_oferty, która dotyczy
-- kontroferty, a jego kontrofertą jest nowa oferta klienta, którą potwierdza jak pierwszą.
--
-- Każda wysyłka oferty klienta i odpowiedzi na kontrofertę czeka na zgodę człowieka, nie wychodzi
-- przed praca od według umowy pośrednictwa ani wtedy, gdy stoi znacznik wątpliwości.
--
-- Oferta nie podaje budżetu klienta ani wyniku analizy kredytowej (R1); treść ustala klient,
-- a pilnuje tego aplikacja. Teczka poszukiwania nie przyjmuje pieniędzy: zadatek i kaucja to tylko
-- warunki oferty.
CREATE TABLE poszukiwanie_oferty_wpisy (
  id TEXT PRIMARY KEY,
  dotyczy TEXT NOT NULL CHECK (dotyczy IN ('kupno', 'najem')),
  kierunek TEXT NOT NULL CHECK (kierunek IN ('od_klienta', 'od_drugiej_strony')),
  propozycja_id TEXT NOT NULL REFERENCES poszukiwanie_propozycje_wpisy (id),
  -- Strona, która sprzedaje albo wynajmuje, i biuro, które ją prowadzi.
  adresat TEXT NOT NULL,
  odpowiada_na_id TEXT REFERENCES poszukiwanie_oferty_wpisy (id),
  -- Wpływ, w którym przyszła kontroferta drugiej strony.
  wplyw_id TEXT REFERENCES sprawa_korespondencja_wplywy (id),
  wpisano TEXT NOT NULL,

  -- Warunki: cena albo czynsz, zadatek albo kaucja, terminy umowy i wydania nieruchomości.
  kwota INTEGER NOT NULL CHECK (kwota > 0),
  zadatek_albo_kaucja INTEGER CHECK (zadatek_albo_kaucja >= 0),
  termin_umowy TEXT,
  termin_wydania TEXT,
  -- Co oferta mówi o źródle pieniędzy, np. „część ceny z kredytu hipotecznego”.
  finansowanie_w_ofercie TEXT,
  warunki_zastrzezone TEXT,

  -- Związanie ofertą (art. 66 § 2 KC): termin z oferty albo sposób złożenia, z którego termin
  -- liczy program.
  wiaze_do TEXT,
  sposob_zlozenia TEXT CHECK (sposob_zlozenia IN ('w_rozmowie', 'na_pismie')),

  -- Zgoda klienta: potwierdzenie treści jego oferty albo jego decyzja o kontrofercie.
  zgoda_id TEXT REFERENCES sprawa_zgody_wpisy (id),

  -- Przebieg oferty klienta: wysyłka i odpowiedź drugiej strony z wpływem i chwilą przedstawienia
  -- klientowi.
  wysylka_id TEXT REFERENCES sprawa_korespondencja_wysylki (id),
  odpowiedz_drugiej_strony TEXT CHECK (odpowiedz_drugiej_strony IN ('przyjeta', 'odrzucona', 'kontroferta')),
  odpowiedz_wplyw_id TEXT REFERENCES sprawa_korespondencja_wplywy (id),
  odpowiedz_przedstawiono TEXT,

  -- Przebieg kontroferty drugiej strony: przedstawienie klientowi, jego decyzja i wysyłka
  -- odpowiedzi.
  przedstawiono TEXT,
  decyzja_klienta TEXT CHECK (decyzja_klienta IN ('przyjeta', 'odrzucona', 'kontroferta')),
  odpowiedz_wysylka_id TEXT REFERENCES sprawa_korespondencja_wysylki (id),

  CONSTRAINT oferta_ma_termin_albo_sposob_zlozenia CHECK (wiaze_do IS NOT NULL OR sposob_zlozenia IS NOT NULL),
  CONSTRAINT kontroferta_drugiej_strony_z_wplywem_w_lancuchu CHECK (
    (kierunek = 'od_drugiej_strony') = (wplyw_id IS NOT NULL)
    AND (kierunek = 'od_klienta' OR odpowiada_na_id IS NOT NULL)
  ),
  CONSTRAINT przebieg_oferty_klienta CHECK (
    kierunek = 'od_klienta'
    OR (wysylka_id IS NULL AND odpowiedz_drugiej_strony IS NULL AND odpowiedz_przedstawiono IS NULL)
  ),
  CONSTRAINT przebieg_kontroferty_drugiej_strony CHECK (
    kierunek = 'od_drugiej_strony'
    OR (przedstawiono IS NULL AND decyzja_klienta IS NULL AND odpowiedz_wysylka_id IS NULL)
  ),
  CONSTRAINT wysylka_po_potwierdzeniu CHECK (wysylka_id IS NULL OR zgoda_id IS NOT NULL),
  CONSTRAINT odpowiedz_drugiej_strony_z_wplywem CHECK (
    (odpowiedz_drugiej_strony IS NULL) = (odpowiedz_wplyw_id IS NULL)
    AND (odpowiedz_drugiej_strony IS NULL OR wysylka_id IS NOT NULL)
    AND (odpowiedz_przedstawiono IS NULL OR odpowiedz_drugiej_strony IS NOT NULL)
  ),
  CONSTRAINT decyzja_po_przedstawieniu CHECK (decyzja_klienta IS NULL OR przedstawiono IS NOT NULL),
  -- Przyjęcie i odrzucenie kontroferty mają zgodę klienta; kontrofertę klienta potwierdza się
  -- na jej własnym wpisie.
  CONSTRAINT decyzja_klienta_ze_zgoda CHECK (
    kierunek = 'od_klienta'
    OR (zgoda_id IS NOT NULL) = coalesce(decyzja_klienta IN ('przyjeta', 'odrzucona'), 0)
  ),
  CONSTRAINT odpowiedz_po_przyjeciu_albo_odrzuceniu CHECK (
    odpowiedz_wysylka_id IS NULL OR decyzja_klienta IN ('przyjeta', 'odrzucona')
  )
) STRICT;

-- Przebieg sprawdza zmiany, więc wpis powstaje bez potwierdzenia, wysyłki, odpowiedzi
-- i decyzji.
CREATE TRIGGER poszukiwanie_oferty_powstaje_bez_przebiegu
BEFORE INSERT ON poszukiwanie_oferty_wpisy
WHEN NEW.zgoda_id IS NOT NULL
  OR NEW.wysylka_id IS NOT NULL
  OR NEW.odpowiedz_drugiej_strony IS NOT NULL
  OR NEW.odpowiedz_przedstawiono IS NOT NULL
  OR NEW.przedstawiono IS NOT NULL
  OR NEW.decyzja_klienta IS NOT NULL
  OR NEW.odpowiedz_wysylka_id IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'oferta powstaje bez potwierdzenia, wysyłki, odpowiedzi i decyzji');
END;

-- Oferta dotyczy propozycji, którą klient dostał, i celu jej profilu: kupna albo najmu.
CREATE TRIGGER poszukiwanie_oferty_dotyczy_przedstawionej_propozycji
BEFORE INSERT ON poszukiwanie_oferty_wpisy
WHEN NOT EXISTS (
  SELECT 1
  FROM poszukiwanie_propozycje_wpisy AS propozycja
  JOIN poszukiwanie_profile_wersje AS profil ON profil.id = propozycja.profil_wersja_id
  WHERE propozycja.id = NEW.propozycja_id
    AND propozycja.przedstawiono IS NOT NULL
    AND profil.cel = NEW.dotyczy
)
BEGIN
  SELECT RAISE(ABORT, 'oferta dotyczy propozycji przedstawionej klientowi i celu jej profilu');
END;

-- Łańcuch: kontroferta drugiej strony odpowiada na ofertę klienta, na którą druga strona
-- odpowiedziała „kontroferta”, a kontroferta klienta na kontrofertę drugiej strony z jego decyzją
-- „kontroferta”; obie dotyczą tej samej propozycji.
CREATE TRIGGER poszukiwanie_oferty_lancuch_kontrofert
BEFORE INSERT ON poszukiwanie_oferty_wpisy
WHEN NEW.odpowiada_na_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM poszukiwanie_oferty_wpisy AS poprzednia
    WHERE poprzednia.id = NEW.odpowiada_na_id
      AND poprzednia.propozycja_id = NEW.propozycja_id
      AND poprzednia.dotyczy = NEW.dotyczy
      AND (
        (NEW.kierunek = 'od_drugiej_strony' AND poprzednia.odpowiedz_drugiej_strony = 'kontroferta')
        OR (NEW.kierunek = 'od_klienta' AND poprzednia.decyzja_klienta = 'kontroferta')
      )
  )
BEGIN
  SELECT RAISE(ABORT, 'kontroferta odpowiada na ofertę tej samej propozycji, na którą druga strona albo klient odpowiedzieli kontrofertą');
END;

-- Treść oferty klienta i jego decyzję o kontrofercie potwierdza klient: zgoda
-- potwierdzenie_oferty uczestnika z obowiązującą rolą klient, która dotyczy tego wpisu.
CREATE TRIGGER poszukiwanie_oferty_zgoda_klienta
BEFORE UPDATE OF zgoda_id ON poszukiwanie_oferty_wpisy
WHEN NEW.zgoda_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_zgody_wpisy AS zgoda
    JOIN sprawa_uczestnicy_role AS rola ON rola.uczestnik_id = zgoda.uczestnik_id
    WHERE zgoda.id = NEW.zgoda_id
      AND zgoda.rodzaj = 'potwierdzenie_oferty'
      AND zgoda.dotyczy_tabela = 'poszukiwanie_oferty_wpisy'
      AND zgoda.dotyczy_id = NEW.id
      AND rola.rola = 'klient'
      AND rola.zakonczono IS NULL
  )
BEGIN
  SELECT RAISE(ABORT, 'oferta klienta i jego decyzja mają zgodę potwierdzenie_oferty klienta, która dotyczy tego wpisu');
END;

-- Oferta i odpowiedź wskazują wysyłkę, która jeszcze nie wyszła, żeby jej wysłanie przeszło
-- przez zgodę.
CREATE TRIGGER poszukiwanie_oferty_wysylka_przed_wyslaniem
BEFORE UPDATE OF wysylka_id, odpowiedz_wysylka_id ON poszukiwanie_oferty_wpisy
WHEN EXISTS (
  SELECT 1
  FROM sprawa_korespondencja_wysylki
  WHERE id IN (NEW.wysylka_id, NEW.odpowiedz_wysylka_id)
    AND id IS NOT OLD.wysylka_id
    AND id IS NOT OLD.odpowiedz_wysylka_id
    AND stan NOT IN ('projekt', 'czeka_na_zgode')
)
BEGIN
  SELECT RAISE(ABORT, 'oferta i odpowiedź wskazują wysyłkę, która jeszcze nie wyszła');
END;

-- Warunków się nie zmienia; potwierdzenie, wysyłkę, odpowiedź drugiej strony, przedstawienie
-- i decyzję klienta wpisuje się raz.
CREATE TRIGGER poszukiwanie_oferty_wpisy_bez_zmian
BEFORE UPDATE ON poszukiwanie_oferty_wpisy
WHEN NEW.id IS NOT OLD.id
  OR NEW.dotyczy IS NOT OLD.dotyczy
  OR NEW.kierunek IS NOT OLD.kierunek
  OR NEW.propozycja_id IS NOT OLD.propozycja_id
  OR NEW.adresat IS NOT OLD.adresat
  OR NEW.odpowiada_na_id IS NOT OLD.odpowiada_na_id
  OR NEW.wplyw_id IS NOT OLD.wplyw_id
  OR NEW.wpisano IS NOT OLD.wpisano
  OR NEW.kwota IS NOT OLD.kwota
  OR NEW.zadatek_albo_kaucja IS NOT OLD.zadatek_albo_kaucja
  OR NEW.termin_umowy IS NOT OLD.termin_umowy
  OR NEW.termin_wydania IS NOT OLD.termin_wydania
  OR NEW.finansowanie_w_ofercie IS NOT OLD.finansowanie_w_ofercie
  OR NEW.warunki_zastrzezone IS NOT OLD.warunki_zastrzezone
  OR NEW.wiaze_do IS NOT OLD.wiaze_do
  OR NEW.sposob_zlozenia IS NOT OLD.sposob_zlozenia
  OR (OLD.zgoda_id IS NOT NULL AND NEW.zgoda_id IS NOT OLD.zgoda_id)
  OR (OLD.wysylka_id IS NOT NULL AND NEW.wysylka_id IS NOT OLD.wysylka_id)
  OR (OLD.odpowiedz_drugiej_strony IS NOT NULL AND NEW.odpowiedz_drugiej_strony IS NOT OLD.odpowiedz_drugiej_strony)
  OR (OLD.odpowiedz_wplyw_id IS NOT NULL AND NEW.odpowiedz_wplyw_id IS NOT OLD.odpowiedz_wplyw_id)
  OR (OLD.odpowiedz_przedstawiono IS NOT NULL AND NEW.odpowiedz_przedstawiono IS NOT OLD.odpowiedz_przedstawiono)
  OR (OLD.przedstawiono IS NOT NULL AND NEW.przedstawiono IS NOT OLD.przedstawiono)
  OR (OLD.decyzja_klienta IS NOT NULL AND NEW.decyzja_klienta IS NOT OLD.decyzja_klienta)
  OR (OLD.odpowiedz_wysylka_id IS NOT NULL AND NEW.odpowiedz_wysylka_id IS NOT OLD.odpowiedz_wysylka_id)
BEGIN
  SELECT RAISE(ABORT, 'oferty się nie zmienia; potwierdzenie, wysyłkę, odpowiedzi i decyzję klienta wpisuje się raz');
END;

-- Wysyłka oferty i odpowiedzi zawsze czeka na zgodę człowieka, także gdy idzie według wzoru
-- biura.
CREATE TRIGGER poszukiwanie_oferty_wysylka_czeka_na_zgode
BEFORE UPDATE OF stan ON sprawa_korespondencja_wysylki
WHEN NEW.stan = 'wyslana'
  AND OLD.stan = 'projekt'
  AND EXISTS (
    SELECT 1
    FROM poszukiwanie_oferty_wpisy
    WHERE NEW.id IN (wysylka_id, odpowiedz_wysylka_id)
  )
BEGIN
  SELECT RAISE(ABORT, 'oferta klienta i odpowiedź przed wysłaniem czekają na zgodę');
END;

-- Oferta wychodzi przy zawartej umowie pośrednictwa, nie wcześniej niż biuro może według niej
-- pracować (art. 15 ust. 3 i art. 21 ust. 2 upk).
CREATE TRIGGER poszukiwanie_oferty_wysylka_od_pracy
BEFORE UPDATE OF stan ON sprawa_korespondencja_wysylki
WHEN NEW.stan = 'wyslana'
  AND EXISTS (
    SELECT 1
    FROM poszukiwanie_oferty_wpisy
    WHERE NEW.id IN (wysylka_id, odpowiedz_wysylka_id)
  )
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_umowy_wpisy AS umowa
    JOIN sprawa_umowy_praca_od AS praca ON praca.umowa_id = umowa.id
    WHERE umowa.rodzaj = 'posrednictwo'
      AND umowa.stan = 'zawarta'
      AND NEW.wyslano >= praca.od
  )
BEGIN
  SELECT RAISE(ABORT, 'oferta klienta wychodzi przy zawartej umowie pośrednictwa, nie wcześniej niż praca od');
END;

-- Oferta i odpowiedź nie wychodzą, dopóki stoi znacznik wątpliwości Kontrolera.
CREATE TRIGGER poszukiwanie_oferty_watpliwosc_blokuje_wysylke
BEFORE UPDATE OF stan ON sprawa_korespondencja_wysylki
WHEN NEW.stan = 'wyslana'
  AND EXISTS (
    SELECT 1
    FROM poszukiwanie_oferty_wpisy
    WHERE NEW.id IN (wysylka_id, odpowiedz_wysylka_id)
  )
  AND EXISTS (
    SELECT 1
    FROM sprawa_metryka_watpliwosci
    WHERE zdjecie_decyzja_id IS NULL
  )
BEGIN
  SELECT RAISE(ABORT, 'dopóki stoi znacznik wątpliwości, oferta klienta i odpowiedź nie wychodzą');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER poszukiwanie_oferty_wpisy_bez_zastepowania
BEFORE INSERT ON poszukiwanie_oferty_wpisy
WHEN EXISTS (
  SELECT 1
  FROM poszukiwanie_oferty_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER poszukiwanie_oferty_wpisy_bez_usuwania
BEFORE DELETE ON poszukiwanie_oferty_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;
"#,
];

/// The values of column `wpisy.dotyczy`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyDotyczy {
    Kupno,
    Najem,
}

impl WpisyDotyczy {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyDotyczy] = &[WpisyDotyczy::Kupno, WpisyDotyczy::Najem];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyDotyczy::Kupno => "kupno",
            WpisyDotyczy::Najem => "najem",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyDotyczy {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "kupno" => Ok(WpisyDotyczy::Kupno),
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
    OdKlienta,
    OdDrugiejStrony,
}

impl WpisyKierunek {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyKierunek] = &[
        WpisyKierunek::OdKlienta,
        WpisyKierunek::OdDrugiejStrony,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyKierunek::OdKlienta => "od_klienta",
            WpisyKierunek::OdDrugiejStrony => "od_drugiej_strony",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyKierunek {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "od_klienta" => Ok(WpisyKierunek::OdKlienta),
                    "od_drugiej_strony" => Ok(WpisyKierunek::OdDrugiejStrony),
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

/// The values of column `wpisy.odpowiedz_drugiej_strony`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyOdpowiedzDrugiejStrony {
    Przyjeta,
    Odrzucona,
    Kontroferta,
}

impl WpisyOdpowiedzDrugiejStrony {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyOdpowiedzDrugiejStrony] = &[
        WpisyOdpowiedzDrugiejStrony::Przyjeta,
        WpisyOdpowiedzDrugiejStrony::Odrzucona,
        WpisyOdpowiedzDrugiejStrony::Kontroferta,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyOdpowiedzDrugiejStrony::Przyjeta => "przyjeta",
            WpisyOdpowiedzDrugiejStrony::Odrzucona => "odrzucona",
            WpisyOdpowiedzDrugiejStrony::Kontroferta => "kontroferta",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyOdpowiedzDrugiejStrony {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "przyjeta" => Ok(WpisyOdpowiedzDrugiejStrony::Przyjeta),
                    "odrzucona" => Ok(WpisyOdpowiedzDrugiejStrony::Odrzucona),
                    "kontroferta" => Ok(WpisyOdpowiedzDrugiejStrony::Kontroferta),
                    other => {
                        Err(
                            format!("{other} is no value of WpisyOdpowiedzDrugiejStrony"),
                        )
                    }
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyOdpowiedzDrugiejStrony {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// A row of `oferty_klienta`.
#[derive(Debug, Clone, PartialEq)]
pub struct OfertyKlientaRow {
    pub id: String,
    pub dotyczy: WpisyDotyczy,
    pub kierunek: WpisyKierunek,
    pub propozycja_id: String,
    pub adresat: String,
    pub odpowiada_na_id: Option<String>,
    pub wplyw_id: Option<String>,
    pub wpisano: String,
    pub kwota: i64,
    pub zadatek_albo_kaucja: Option<i64>,
    pub termin_umowy: Option<String>,
    pub termin_wydania: Option<String>,
    pub finansowanie_w_ofercie: Option<String>,
    pub warunki_zastrzezone: Option<String>,
    pub wiaze_do: Option<String>,
    pub sposob_zlozenia: Option<WpisySposobZlozenia>,
    pub zgoda_id: Option<String>,
    pub wysylka_id: Option<String>,
    pub odpowiedz_drugiej_strony: Option<WpisyOdpowiedzDrugiejStrony>,
    pub odpowiedz_wplyw_id: Option<String>,
    pub odpowiedz_przedstawiono: Option<String>,
    pub przedstawiono: Option<String>,
    pub decyzja_klienta: Option<WpisyOdpowiedzDrugiejStrony>,
    pub odpowiedz_wysylka_id: Option<String>,
}

impl OfertyKlientaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(OfertyKlientaRow {
            id: row.get("id")?,
            dotyczy: row.get("dotyczy")?,
            kierunek: row.get("kierunek")?,
            propozycja_id: row.get("propozycja_id")?,
            adresat: row.get("adresat")?,
            odpowiada_na_id: row.get("odpowiada_na_id")?,
            wplyw_id: row.get("wplyw_id")?,
            wpisano: row.get("wpisano")?,
            kwota: row.get("kwota")?,
            zadatek_albo_kaucja: row.get("zadatek_albo_kaucja")?,
            termin_umowy: row.get("termin_umowy")?,
            termin_wydania: row.get("termin_wydania")?,
            finansowanie_w_ofercie: row.get("finansowanie_w_ofercie")?,
            warunki_zastrzezone: row.get("warunki_zastrzezone")?,
            wiaze_do: row.get("wiaze_do")?,
            sposob_zlozenia: row.get("sposob_zlozenia")?,
            zgoda_id: row.get("zgoda_id")?,
            wysylka_id: row.get("wysylka_id")?,
            odpowiedz_drugiej_strony: row.get("odpowiedz_drugiej_strony")?,
            odpowiedz_wplyw_id: row.get("odpowiedz_wplyw_id")?,
            odpowiedz_przedstawiono: row.get("odpowiedz_przedstawiono")?,
            przedstawiono: row.get("przedstawiono")?,
            decyzja_klienta: row.get("decyzja_klienta")?,
            odpowiedz_wysylka_id: row.get("odpowiedz_wysylka_id")?,
        })
    }
}

/// Oferty klienta i kontroferty drugiej strony w kolejności wpisania.
pub fn oferty_klienta(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<OfertyKlientaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM poszukiwanie_oferty_wpisy
ORDER BY wpisano, rowid"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(OfertyKlientaRow::from_row).collect()
}

/// The parameters of `przygotuj_oferte_klienta`.
#[derive(Debug, Clone, PartialEq)]
pub struct PrzygotujOferteKlientaParams {
    pub id: String,
    pub dotyczy: WpisyDotyczy,
    pub propozycja_id: String,
    pub adresat: String,
    pub odpowiada_na_id: Option<String>,
    pub wpisano: String,
    pub kwota: i64,
    pub zadatek_albo_kaucja: Option<i64>,
    pub termin_umowy: Option<String>,
    pub termin_wydania: Option<String>,
    pub finansowanie_w_ofercie: Option<String>,
    pub warunki_zastrzezone: Option<String>,
    pub wiaze_do: Option<String>,
    pub sposob_zlozenia: Option<WpisySposobZlozenia>,
}

/// Oferta albo kontroferta klienta według wzoru biura, przed jego potwierdzeniem; kontroferta
/// wskazuje kontrofertę drugiej strony z decyzją klienta „kontroferta”. Identyfikator (UUID)
/// nadaje aplikacja, bo wskazuje go zgoda klienta.
pub fn przygotuj_oferte_klienta(
    room: &impl xmeta_room::MetaRoom,
    params: &PrzygotujOferteKlientaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO poszukiwanie_oferty_wpisy (id, dotyczy, kierunek, propozycja_id, adresat,
  odpowiada_na_id, wpisano, kwota, zadatek_albo_kaucja, termin_umowy, termin_wydania,
  finansowanie_w_ofercie, warunki_zastrzezone, wiaze_do, sposob_zlozenia)
VALUES (:id, :dotyczy, 'od_klienta', :propozycja_id, :adresat, :odpowiada_na_id, :wpisano, :kwota,
  :zadatek_albo_kaucja, :termin_umowy, :termin_wydania, :finansowanie_w_ofercie,
  :warunki_zastrzezone, :wiaze_do, :sposob_zlozenia)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("dotyczy", xmeta_room::ToSqlValue::to_sql_value(&params.dotyczy)),
                    (
                        "propozycja_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.propozycja_id),
                    ),
                    ("adresat", xmeta_room::ToSqlValue::to_sql_value(&params.adresat)),
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
                        "finansowanie_w_ofercie",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.finansowanie_w_ofercie,
                        ),
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

/// The parameters of `potwierdz_oferte_klienta`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PotwierdzOferteKlientaParams {
    pub zgoda_id: String,
    pub id: String,
}

/// Klient potwierdził treść swojej oferty zgodą potwierdzenie_oferty.
pub fn potwierdz_oferte_klienta(
    room: &impl xmeta_room::MetaRoom,
    params: &PotwierdzOferteKlientaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE poszukiwanie_oferty_wpisy
SET zgoda_id = :zgoda_id
WHERE id = :id"#,
                parameters: Vec::from([
                    ("zgoda_id", xmeta_room::ToSqlValue::to_sql_value(&params.zgoda_id)),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `wskaz_wysylke_oferty_klienta`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WskazWysylkeOfertyKlientaParams {
    pub wysylka_id: String,
    pub id: String,
}

/// Wysyłka potwierdzonej oferty klienta; jeszcze niewysłana, bo wychodzi tylko za zgodą
/// człowieka.
pub fn wskaz_wysylke_oferty_klienta(
    room: &impl xmeta_room::MetaRoom,
    params: &WskazWysylkeOfertyKlientaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE poszukiwanie_oferty_wpisy
SET wysylka_id = :wysylka_id
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "wysylka_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wysylka_id),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_odpowiedz_drugiej_strony`.
#[derive(Debug, Clone, PartialEq)]
pub struct OdnotujOdpowiedzDrugiejStronyParams {
    pub odpowiedz_drugiej_strony: WpisyOdpowiedzDrugiejStrony,
    pub odpowiedz_wplyw_id: String,
    pub odpowiedz_przedstawiono: Option<String>,
    pub id: String,
}

/// Przyjęcie, odrzucenie albo kontroferta drugiej strony z wpływem i chwilą, w której klient je
/// poznał.
pub fn odnotuj_odpowiedz_drugiej_strony(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujOdpowiedzDrugiejStronyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE poszukiwanie_oferty_wpisy
SET odpowiedz_drugiej_strony = :odpowiedz_drugiej_strony,
  odpowiedz_wplyw_id = :odpowiedz_wplyw_id,
  odpowiedz_przedstawiono = :odpowiedz_przedstawiono
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
                    (
                        "odpowiedz_przedstawiono",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.odpowiedz_przedstawiono,
                        ),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `wpisz_kontroferte_drugiej_strony`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszKontroferteDrugiejStronyParams {
    pub id: String,
    pub dotyczy: WpisyDotyczy,
    pub propozycja_id: String,
    pub adresat: String,
    pub odpowiada_na_id: String,
    pub wplyw_id: String,
    pub wpisano: String,
    pub kwota: i64,
    pub zadatek_albo_kaucja: Option<i64>,
    pub termin_umowy: Option<String>,
    pub termin_wydania: Option<String>,
    pub finansowanie_w_ofercie: Option<String>,
    pub warunki_zastrzezone: Option<String>,
    pub wiaze_do: Option<String>,
    pub sposob_zlozenia: Option<WpisySposobZlozenia>,
}

/// Kontroferta drugiej strony z wpływem, która odpowiada na ofertę klienta.
pub fn wpisz_kontroferte_drugiej_strony(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszKontroferteDrugiejStronyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO poszukiwanie_oferty_wpisy (id, dotyczy, kierunek, propozycja_id, adresat,
  odpowiada_na_id, wplyw_id, wpisano, kwota, zadatek_albo_kaucja, termin_umowy, termin_wydania,
  finansowanie_w_ofercie, warunki_zastrzezone, wiaze_do, sposob_zlozenia)
VALUES (:id, :dotyczy, 'od_drugiej_strony', :propozycja_id, :adresat, :odpowiada_na_id, :wplyw_id,
  :wpisano, :kwota, :zadatek_albo_kaucja, :termin_umowy, :termin_wydania, :finansowanie_w_ofercie,
  :warunki_zastrzezone, :wiaze_do, :sposob_zlozenia)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("dotyczy", xmeta_room::ToSqlValue::to_sql_value(&params.dotyczy)),
                    (
                        "propozycja_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.propozycja_id),
                    ),
                    ("adresat", xmeta_room::ToSqlValue::to_sql_value(&params.adresat)),
                    (
                        "odpowiada_na_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.odpowiada_na_id),
                    ),
                    ("wplyw_id", xmeta_room::ToSqlValue::to_sql_value(&params.wplyw_id)),
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
                        "finansowanie_w_ofercie",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.finansowanie_w_ofercie,
                        ),
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

/// The parameters of `odnotuj_przedstawienie_kontroferty`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujPrzedstawienieKontrofertyParams {
    pub przedstawiono: String,
    pub id: String,
}

/// Chwila, w której klient dostał kontrofertę drugiej strony.
pub fn odnotuj_przedstawienie_kontroferty(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujPrzedstawienieKontrofertyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE poszukiwanie_oferty_wpisy
SET przedstawiono = :przedstawiono
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "przedstawiono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.przedstawiono),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_decyzje_klienta_o_kontrofercie`.
#[derive(Debug, Clone, PartialEq)]
pub struct OdnotujDecyzjeKlientaOKontrofercieParams {
    pub decyzja_klienta: WpisyOdpowiedzDrugiejStrony,
    pub zgoda_id: Option<String>,
    pub id: String,
}

/// Przyjęcie albo odrzucenie kontroferty ze zgodą potwierdzenie_oferty albo kontroferta klienta,
/// którą klient potwierdza na jej własnym wpisie.
pub fn odnotuj_decyzje_klienta_o_kontrofercie(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujDecyzjeKlientaOKontrofercieParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE poszukiwanie_oferty_wpisy
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

/// The parameters of `wskaz_odpowiedz_na_kontroferte`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WskazOdpowiedzNaKontroferteParams {
    pub odpowiedz_wysylka_id: String,
    pub id: String,
}

/// Wysyłka z przyjęciem albo odrzuceniem kontroferty; jeszcze niewysłana, bo wychodzi tylko za
/// zgodą człowieka.
pub fn wskaz_odpowiedz_na_kontroferte(
    room: &impl xmeta_room::MetaRoom,
    params: &WskazOdpowiedzNaKontroferteParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE poszukiwanie_oferty_wpisy
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
