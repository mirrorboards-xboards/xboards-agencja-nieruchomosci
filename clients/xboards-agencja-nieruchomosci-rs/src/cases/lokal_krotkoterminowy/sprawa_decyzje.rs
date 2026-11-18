//! The kind `sprawa-decyzje` of the room `cases/lokal_krotkoterminowy`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind: its name, the kinds it requires and its migrations in the order of their
/// numbers.
pub const KIND: xmeta_room::Kind = xmeta_room::Kind {
    name: "sprawa-decyzje",
    requires: &[],
    migrations: &[
        r#"-- Prośby agentów o decyzję człowieka i rozstrzygnięcia z podpisem. Inne wiersze teczki (i typy
-- spraw) powołują się na decyzję, więc decyzja jest pierwszym rodzajem rdzenia: wskazują na nią
-- wszystkie późniejsze.
--
-- Prośbę składa agent sprawy albo Kontroler. Rozstrzyga człowiek: zwykle właściciel agenta,
-- a prośby Kontrolera właściciel Kierownika kontroli. Decyzja „zatwierdzona” to rozstrzygnięcie
-- „zgoda” albo „zgoda z warunkiem”; warunku pilnują ludzie, nie baza.
--
-- Podpis obejmuje identyfikator, rodzaj, wskazany wiersz, rozstrzygnięcie, czas i skrót wiersza
-- z plikiem (dotyczy_skrot), który liczy aplikacja. Baza wymaga kompletu kolumn podpisu, a sam
-- podpis kluczem xkeys sprawdza aplikacja. Decyzji się nie zmienia: nowa decyzja w tej samej
-- sprawie zastępuje poprzednią i ją wskazuje (zastepuje_decyzja_id). Gdy nowa decyzja zostaje
-- rozstrzygnięta, poprzednia przechodzi w stan „zastapiona”. Reguły innych rodzajów (rdzenia
-- i typów spraw), które wymagają zatwierdzonej decyzji, czytają widok sprawa_decyzje_zatwierdzone.
-- Liczy on tylko stan „rozstrzygnieta”, więc zastąpiona decyzja przestaje im wystarczać.
--
-- Reguły, które zależą od późniejszych rodzajów, leżą w ich migracjach:
-- - znacznik wątpliwości blokuje zgodę na termin, rachunek i wypłatę: sprawa-metryka
--   (sprawa_metryka_watpliwosc_blokuje_zgode);
-- - zgoda na rachunek wymaga sprawdzenia i potwierdzenia: sprawa-sprawdzenia
--   (sprawa_sprawdzenia_dopuszczenie_rachunku_wymaga_sprawdzenia);
-- - zgoda na wypłatę zapada po sprawdzeniu rachunku: sprawa-wyplaty
--   (sprawa_wyplaty_zgoda_po_sprawdzeniu_rachunku);
-- - o zgodę na wysyłkę prosi się, gdy wysyłka czeka na zgodę: sprawa-korespondencja
--   (sprawa_korespondencja_prosba_o_wyslanie_przy_czekajacej).
CREATE TABLE sprawa_decyzje_wpisy (
  id TEXT PRIMARY KEY,
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('potwierdzenie_terminu', 'dopuszczenie_rachunku', 'wyplata', 'wyslanie', 'zdjecie_watpliwosci', 'przerwanie_sprawy', 'wznowienie_sprawy', 'inna')),
  prosi TEXT NOT NULL,
  -- Stanowiska: nazwy ich folderów pod positions/.
  prosi_stanowisko TEXT NOT NULL CHECK (prosi_stanowisko IN ('analityk_kredytowy', 'analityk_stanu_prawnego', 'kierownik_biura', 'kierownik_kontroli', 'kierownik_najmu', 'kierownik_posrednictwa', 'kierownik_transakcji', 'konsjerz', 'kontroler', 'koordynator_operacji', 'koordynator_transakcji', 'opiekun_najmu', 'opiekun_oferty', 'opiekun_poszukujacego', 'recepcja', 'zarzadca_obiektu')),
  -- Człowiek, który ma rozstrzygnąć, i do kiedy.
  decyduje TEXT NOT NULL,
  do_kiedy TEXT,
  zlozono TEXT NOT NULL,
  -- Czego prośba dotyczy, krótko; treści wskazanego wiersza się nie powtarza.
  tresc TEXT,
  -- Wiersz, którego dotyczy decyzja: tabela teczki (rdzenia albo typu sprawy) i identyfikator.
  -- Bez klucza obcego, bo może to być wiersz dowolnej tabeli, także typu sprawy.
  dotyczy_tabela TEXT,
  dotyczy_id TEXT,
  dotyczy_skrot TEXT,
  stan TEXT NOT NULL CHECK (stan IN ('czeka', 'rozstrzygnieta', 'zastapiona', 'zbedna')),
  rozstrzygniecie TEXT CHECK (rozstrzygniecie IN ('zgoda', 'odmowa', 'zgoda_z_warunkiem')),
  warunek TEXT,
  uzasadnienie TEXT,
  podpisal TEXT,
  -- Człowiek, za którego rozstrzyga zastępca; NULL, gdy rozstrzyga sam wskazany.
  za_kogo TEXT,
  podpis TEXT,
  podpisano TEXT,
  zastepuje_decyzja_id TEXT REFERENCES sprawa_decyzje_wpisy (id),
  powod_zbednosci TEXT,

  CONSTRAINT podpis_w_komplecie CHECK (
    (podpisal IS NULL) = (rozstrzygniecie IS NULL)
    AND (podpisal IS NULL) = (podpis IS NULL)
    AND (podpisal IS NULL) = (podpisano IS NULL)
    AND (za_kogo IS NULL OR podpisal IS NOT NULL)
  ),
  -- Zastąpiona decyzja była rozstrzygnięta i zachowuje swój podpis.
  CONSTRAINT podpis_tylko_przy_rozstrzygnieciu CHECK ((stan IN ('rozstrzygnieta', 'zastapiona')) = (podpisal IS NOT NULL)),
  CONSTRAINT rozstrzygniecie_nie_przed_zlozeniem CHECK (podpisano IS NULL OR podpisano >= zlozono),
  CONSTRAINT dotyczy_tabela_razem_z_id CHECK ((dotyczy_tabela IS NULL) = (dotyczy_id IS NULL)),
  CONSTRAINT skrot_tylko_przy_wierszu CHECK (dotyczy_skrot IS NULL OR dotyczy_id IS NOT NULL),
  CONSTRAINT zgoda_z_warunkiem_ma_warunek CHECK ((rozstrzygniecie IS 'zgoda_z_warunkiem') = (warunek IS NOT NULL)),
  CONSTRAINT zbedna_ma_powod CHECK ((stan = 'zbedna') = (powod_zbednosci IS NOT NULL)),
  -- Art. 54 ust. 1 AML: teczkę czytają osoby obsługujące klienta, więc decyzja o zdjęciu
  -- wątpliwości nie niesie w teczce żadnej treści. Podstawy są w rejestrze kontroli.
  CONSTRAINT decyzja_aml_bez_tresci CHECK (
    rodzaj <> 'zdjecie_watpliwosci'
    OR (tresc IS NULL AND uzasadnienie IS NULL AND warunek IS NULL)
  )
) STRICT;

-- Zatwierdzone decyzje: rozstrzygnięte zgodą albo zgodą z warunkiem. To jedyne miejsce tej reguły;
-- korzystają z niego reguły rdzenia i typów spraw, które wymagają zatwierdzonej decyzji. Decyzja
-- zastąpiona nowszą ma stan „zastapiona”, więc z widoku wypada.
CREATE VIEW sprawa_decyzje_zatwierdzone AS
SELECT *
FROM sprawa_decyzje_wpisy
WHERE stan = 'rozstrzygnieta'
  AND rozstrzygniecie IN ('zgoda', 'zgoda_z_warunkiem');

-- Reguły rozstrzygnięcia sprawdzają zmianę stanu, więc prośba nie może powstać od razu
-- rozstrzygnięta.
CREATE TRIGGER sprawa_decyzje_prosba_powstaje_jako_czekajaca
BEFORE INSERT ON sprawa_decyzje_wpisy
WHEN NEW.stan <> 'czeka'
BEGIN
  SELECT RAISE(ABORT, 'prośba o decyzję powstaje jako czekająca');
END;

-- Na decyzję powołują się inne wiersze, a jej podpis obejmuje rozstrzygnięcie i skrót wiersza.
-- Jedyny wyjątek to przejście rozstrzygniętej decyzji w stan „zastapiona”, bez zmiany innych
-- kolumn, gdy istnieje rozstrzygnięta decyzja, która ją zastępuje.
CREATE TRIGGER sprawa_decyzje_rozstrzygniecie_jest_ostateczne
BEFORE UPDATE ON sprawa_decyzje_wpisy
WHEN OLD.stan <> 'czeka'
  AND NOT (
    OLD.stan = 'rozstrzygnieta'
    AND NEW.stan = 'zastapiona'
    AND NEW.id IS OLD.id
    AND NEW.rodzaj IS OLD.rodzaj
    AND NEW.prosi IS OLD.prosi
    AND NEW.prosi_stanowisko IS OLD.prosi_stanowisko
    AND NEW.decyduje IS OLD.decyduje
    AND NEW.do_kiedy IS OLD.do_kiedy
    AND NEW.zlozono IS OLD.zlozono
    AND NEW.tresc IS OLD.tresc
    AND NEW.dotyczy_tabela IS OLD.dotyczy_tabela
    AND NEW.dotyczy_id IS OLD.dotyczy_id
    AND NEW.dotyczy_skrot IS OLD.dotyczy_skrot
    AND NEW.rozstrzygniecie IS OLD.rozstrzygniecie
    AND NEW.warunek IS OLD.warunek
    AND NEW.uzasadnienie IS OLD.uzasadnienie
    AND NEW.podpisal IS OLD.podpisal
    AND NEW.za_kogo IS OLD.za_kogo
    AND NEW.podpis IS OLD.podpis
    AND NEW.podpisano IS OLD.podpisano
    AND NEW.zastepuje_decyzja_id IS OLD.zastepuje_decyzja_id
    AND NEW.powod_zbednosci IS OLD.powod_zbednosci
    AND EXISTS (
      SELECT 1
      FROM sprawa_decyzje_wpisy AS nowsza
      WHERE nowsza.zastepuje_decyzja_id = OLD.id
        AND nowsza.stan = 'rozstrzygnieta'
    )
  )
BEGIN
  SELECT RAISE(ABORT, 'rozstrzygniętej ani zbędnej decyzji się nie zmienia; nowa decyzja zastępuje poprzednią');
END;

-- Zastępuje się tylko obowiązującą decyzję o tej samej sprawie: tego samego rodzaju i o tym samym
-- wierszu. Inaczej nowa decyzja odebrałaby moc decyzji, której nie dotyczy.
CREATE TRIGGER sprawa_decyzje_zastepuje_obowiazujaca_o_tym_samym
BEFORE INSERT ON sprawa_decyzje_wpisy
WHEN NEW.zastepuje_decyzja_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_decyzje_wpisy AS poprzednia
    WHERE poprzednia.id = NEW.zastepuje_decyzja_id
      AND poprzednia.stan = 'rozstrzygnieta'
      AND poprzednia.rodzaj = NEW.rodzaj
      AND poprzednia.dotyczy_tabela IS NEW.dotyczy_tabela
      AND poprzednia.dotyczy_id IS NEW.dotyczy_id
  )
BEGIN
  SELECT RAISE(ABORT, 'nowa decyzja zastępuje obowiązującą rozstrzygniętą decyzję tego samego rodzaju o tym samym wierszu');
END;

-- To samo w chwili rozstrzygnięcia: między prośbą a rozstrzygnięciem poprzednią decyzję mogła
-- już zastąpić inna.
CREATE TRIGGER sprawa_decyzje_rozstrzygnieta_zastepuje_obowiazujaca_o_tym_samym
BEFORE UPDATE OF stan ON sprawa_decyzje_wpisy
WHEN NEW.stan = 'rozstrzygnieta'
  AND OLD.stan = 'czeka'
  AND NEW.zastepuje_decyzja_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_decyzje_wpisy AS poprzednia
    WHERE poprzednia.id = NEW.zastepuje_decyzja_id
      AND poprzednia.stan = 'rozstrzygnieta'
      AND poprzednia.rodzaj = NEW.rodzaj
      AND poprzednia.dotyczy_tabela IS NEW.dotyczy_tabela
      AND poprzednia.dotyczy_id IS NEW.dotyczy_id
  )
BEGIN
  SELECT RAISE(ABORT, 'nowa decyzja zastępuje obowiązującą rozstrzygniętą decyzję tego samego rodzaju o tym samym wierszu');
END;

-- Rozstrzygnięcie nowej decyzji odbiera moc poprzedniej, także gdy nowa jest odmową.
CREATE TRIGGER sprawa_decyzje_rozstrzygnieta_zastepuje_poprzednia
AFTER UPDATE OF stan ON sprawa_decyzje_wpisy
WHEN NEW.stan = 'rozstrzygnieta'
  AND OLD.stan = 'czeka'
  AND NEW.zastepuje_decyzja_id IS NOT NULL
BEGIN
  UPDATE sprawa_decyzje_wpisy
  SET stan = 'zastapiona'
  WHERE id = NEW.zastepuje_decyzja_id;
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER sprawa_decyzje_wpisy_bez_zastepowania
BEFORE INSERT ON sprawa_decyzje_wpisy
WHEN EXISTS (
  SELECT 1
  FROM sprawa_decyzje_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER sprawa_decyzje_wpisy_bez_usuwania
BEFORE DELETE ON sprawa_decyzje_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;
"#,
    ],
};

/// The values of column `wpisy.rodzaj`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyRodzaj {
    PotwierdzenieTerminu,
    DopuszczenieRachunku,
    Wyplata,
    Wyslanie,
    ZdjecieWatpliwosci,
    PrzerwanieSprawy,
    WznowienieSprawy,
    Inna,
}

impl WpisyRodzaj {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyRodzaj] = &[
        WpisyRodzaj::PotwierdzenieTerminu,
        WpisyRodzaj::DopuszczenieRachunku,
        WpisyRodzaj::Wyplata,
        WpisyRodzaj::Wyslanie,
        WpisyRodzaj::ZdjecieWatpliwosci,
        WpisyRodzaj::PrzerwanieSprawy,
        WpisyRodzaj::WznowienieSprawy,
        WpisyRodzaj::Inna,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyRodzaj::PotwierdzenieTerminu => "potwierdzenie_terminu",
            WpisyRodzaj::DopuszczenieRachunku => "dopuszczenie_rachunku",
            WpisyRodzaj::Wyplata => "wyplata",
            WpisyRodzaj::Wyslanie => "wyslanie",
            WpisyRodzaj::ZdjecieWatpliwosci => "zdjecie_watpliwosci",
            WpisyRodzaj::PrzerwanieSprawy => "przerwanie_sprawy",
            WpisyRodzaj::WznowienieSprawy => "wznowienie_sprawy",
            WpisyRodzaj::Inna => "inna",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyRodzaj {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "potwierdzenie_terminu" => Ok(WpisyRodzaj::PotwierdzenieTerminu),
                    "dopuszczenie_rachunku" => Ok(WpisyRodzaj::DopuszczenieRachunku),
                    "wyplata" => Ok(WpisyRodzaj::Wyplata),
                    "wyslanie" => Ok(WpisyRodzaj::Wyslanie),
                    "zdjecie_watpliwosci" => Ok(WpisyRodzaj::ZdjecieWatpliwosci),
                    "przerwanie_sprawy" => Ok(WpisyRodzaj::PrzerwanieSprawy),
                    "wznowienie_sprawy" => Ok(WpisyRodzaj::WznowienieSprawy),
                    "inna" => Ok(WpisyRodzaj::Inna),
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

/// The values of column `wpisy.prosi_stanowisko`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyProsiStanowisko {
    AnalitykKredytowy,
    AnalitykStanuPrawnego,
    KierownikBiura,
    KierownikKontroli,
    KierownikNajmu,
    KierownikPosrednictwa,
    KierownikTransakcji,
    Konsjerz,
    Kontroler,
    KoordynatorOperacji,
    KoordynatorTransakcji,
    OpiekunNajmu,
    OpiekunOferty,
    OpiekunPoszukujacego,
    Recepcja,
    ZarzadcaObiektu,
}

impl WpisyProsiStanowisko {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyProsiStanowisko] = &[
        WpisyProsiStanowisko::AnalitykKredytowy,
        WpisyProsiStanowisko::AnalitykStanuPrawnego,
        WpisyProsiStanowisko::KierownikBiura,
        WpisyProsiStanowisko::KierownikKontroli,
        WpisyProsiStanowisko::KierownikNajmu,
        WpisyProsiStanowisko::KierownikPosrednictwa,
        WpisyProsiStanowisko::KierownikTransakcji,
        WpisyProsiStanowisko::Konsjerz,
        WpisyProsiStanowisko::Kontroler,
        WpisyProsiStanowisko::KoordynatorOperacji,
        WpisyProsiStanowisko::KoordynatorTransakcji,
        WpisyProsiStanowisko::OpiekunNajmu,
        WpisyProsiStanowisko::OpiekunOferty,
        WpisyProsiStanowisko::OpiekunPoszukujacego,
        WpisyProsiStanowisko::Recepcja,
        WpisyProsiStanowisko::ZarzadcaObiektu,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyProsiStanowisko::AnalitykKredytowy => "analityk_kredytowy",
            WpisyProsiStanowisko::AnalitykStanuPrawnego => "analityk_stanu_prawnego",
            WpisyProsiStanowisko::KierownikBiura => "kierownik_biura",
            WpisyProsiStanowisko::KierownikKontroli => "kierownik_kontroli",
            WpisyProsiStanowisko::KierownikNajmu => "kierownik_najmu",
            WpisyProsiStanowisko::KierownikPosrednictwa => "kierownik_posrednictwa",
            WpisyProsiStanowisko::KierownikTransakcji => "kierownik_transakcji",
            WpisyProsiStanowisko::Konsjerz => "konsjerz",
            WpisyProsiStanowisko::Kontroler => "kontroler",
            WpisyProsiStanowisko::KoordynatorOperacji => "koordynator_operacji",
            WpisyProsiStanowisko::KoordynatorTransakcji => "koordynator_transakcji",
            WpisyProsiStanowisko::OpiekunNajmu => "opiekun_najmu",
            WpisyProsiStanowisko::OpiekunOferty => "opiekun_oferty",
            WpisyProsiStanowisko::OpiekunPoszukujacego => "opiekun_poszukujacego",
            WpisyProsiStanowisko::Recepcja => "recepcja",
            WpisyProsiStanowisko::ZarzadcaObiektu => "zarzadca_obiektu",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyProsiStanowisko {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "analityk_kredytowy" => Ok(WpisyProsiStanowisko::AnalitykKredytowy),
                    "analityk_stanu_prawnego" => {
                        Ok(WpisyProsiStanowisko::AnalitykStanuPrawnego)
                    }
                    "kierownik_biura" => Ok(WpisyProsiStanowisko::KierownikBiura),
                    "kierownik_kontroli" => Ok(WpisyProsiStanowisko::KierownikKontroli),
                    "kierownik_najmu" => Ok(WpisyProsiStanowisko::KierownikNajmu),
                    "kierownik_posrednictwa" => {
                        Ok(WpisyProsiStanowisko::KierownikPosrednictwa)
                    }
                    "kierownik_transakcji" => {
                        Ok(WpisyProsiStanowisko::KierownikTransakcji)
                    }
                    "konsjerz" => Ok(WpisyProsiStanowisko::Konsjerz),
                    "kontroler" => Ok(WpisyProsiStanowisko::Kontroler),
                    "koordynator_operacji" => {
                        Ok(WpisyProsiStanowisko::KoordynatorOperacji)
                    }
                    "koordynator_transakcji" => {
                        Ok(WpisyProsiStanowisko::KoordynatorTransakcji)
                    }
                    "opiekun_najmu" => Ok(WpisyProsiStanowisko::OpiekunNajmu),
                    "opiekun_oferty" => Ok(WpisyProsiStanowisko::OpiekunOferty),
                    "opiekun_poszukujacego" => {
                        Ok(WpisyProsiStanowisko::OpiekunPoszukujacego)
                    }
                    "recepcja" => Ok(WpisyProsiStanowisko::Recepcja),
                    "zarzadca_obiektu" => Ok(WpisyProsiStanowisko::ZarzadcaObiektu),
                    other => Err(format!("{other} is no value of WpisyProsiStanowisko")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyProsiStanowisko {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wpisy.stan`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyStan {
    Czeka,
    Rozstrzygnieta,
    Zastapiona,
    Zbedna,
}

impl WpisyStan {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyStan] = &[
        WpisyStan::Czeka,
        WpisyStan::Rozstrzygnieta,
        WpisyStan::Zastapiona,
        WpisyStan::Zbedna,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyStan::Czeka => "czeka",
            WpisyStan::Rozstrzygnieta => "rozstrzygnieta",
            WpisyStan::Zastapiona => "zastapiona",
            WpisyStan::Zbedna => "zbedna",
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
                    "zastapiona" => Ok(WpisyStan::Zastapiona),
                    "zbedna" => Ok(WpisyStan::Zbedna),
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
    Zgoda,
    Odmowa,
    ZgodaZWarunkiem,
}

impl WpisyRozstrzygniecie {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyRozstrzygniecie] = &[
        WpisyRozstrzygniecie::Zgoda,
        WpisyRozstrzygniecie::Odmowa,
        WpisyRozstrzygniecie::ZgodaZWarunkiem,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyRozstrzygniecie::Zgoda => "zgoda",
            WpisyRozstrzygniecie::Odmowa => "odmowa",
            WpisyRozstrzygniecie::ZgodaZWarunkiem => "zgoda_z_warunkiem",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyRozstrzygniecie {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "zgoda" => Ok(WpisyRozstrzygniecie::Zgoda),
                    "odmowa" => Ok(WpisyRozstrzygniecie::Odmowa),
                    "zgoda_z_warunkiem" => Ok(WpisyRozstrzygniecie::ZgodaZWarunkiem),
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

/// A row of `decyzje`.
#[derive(Debug, Clone, PartialEq)]
pub struct DecyzjeRow {
    pub id: String,
    pub rodzaj: WpisyRodzaj,
    pub prosi: String,
    pub prosi_stanowisko: WpisyProsiStanowisko,
    pub decyduje: String,
    pub do_kiedy: Option<String>,
    pub zlozono: String,
    pub tresc: Option<String>,
    pub dotyczy_tabela: Option<String>,
    pub dotyczy_id: Option<String>,
    pub dotyczy_skrot: Option<String>,
    pub stan: WpisyStan,
    pub rozstrzygniecie: Option<WpisyRozstrzygniecie>,
    pub warunek: Option<String>,
    pub uzasadnienie: Option<String>,
    pub podpisal: Option<String>,
    pub za_kogo: Option<String>,
    pub podpis: Option<String>,
    pub podpisano: Option<String>,
    pub zastepuje_decyzja_id: Option<String>,
    pub powod_zbednosci: Option<String>,
}

impl DecyzjeRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(DecyzjeRow {
            id: row.get("id")?,
            rodzaj: row.get("rodzaj")?,
            prosi: row.get("prosi")?,
            prosi_stanowisko: row.get("prosi_stanowisko")?,
            decyduje: row.get("decyduje")?,
            do_kiedy: row.get("do_kiedy")?,
            zlozono: row.get("zlozono")?,
            tresc: row.get("tresc")?,
            dotyczy_tabela: row.get("dotyczy_tabela")?,
            dotyczy_id: row.get("dotyczy_id")?,
            dotyczy_skrot: row.get("dotyczy_skrot")?,
            stan: row.get("stan")?,
            rozstrzygniecie: row.get("rozstrzygniecie")?,
            warunek: row.get("warunek")?,
            uzasadnienie: row.get("uzasadnienie")?,
            podpisal: row.get("podpisal")?,
            za_kogo: row.get("za_kogo")?,
            podpis: row.get("podpis")?,
            podpisano: row.get("podpisano")?,
            zastepuje_decyzja_id: row.get("zastepuje_decyzja_id")?,
            powod_zbednosci: row.get("powod_zbednosci")?,
        })
    }
}

/// Wszystkie prośby o decyzję i rozstrzygnięcia sprawy, najnowsze pierwsze.
pub fn decyzje(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<DecyzjeRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM sprawa_decyzje_wpisy
ORDER BY zlozono DESC, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(DecyzjeRow::from_row).collect()
}

/// A row of `decyzje_czekajace`.
#[derive(Debug, Clone, PartialEq)]
pub struct DecyzjeCzekajaceRow {
    pub id: String,
    pub rodzaj: WpisyRodzaj,
    pub prosi: String,
    pub prosi_stanowisko: WpisyProsiStanowisko,
    pub decyduje: String,
    pub do_kiedy: Option<String>,
    pub zlozono: String,
    pub tresc: Option<String>,
    pub dotyczy_tabela: Option<String>,
    pub dotyczy_id: Option<String>,
    pub dotyczy_skrot: Option<String>,
    pub stan: WpisyStan,
    pub rozstrzygniecie: Option<WpisyRozstrzygniecie>,
    pub warunek: Option<String>,
    pub uzasadnienie: Option<String>,
    pub podpisal: Option<String>,
    pub za_kogo: Option<String>,
    pub podpis: Option<String>,
    pub podpisano: Option<String>,
    pub zastepuje_decyzja_id: Option<String>,
    pub powod_zbednosci: Option<String>,
}

impl DecyzjeCzekajaceRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(DecyzjeCzekajaceRow {
            id: row.get("id")?,
            rodzaj: row.get("rodzaj")?,
            prosi: row.get("prosi")?,
            prosi_stanowisko: row.get("prosi_stanowisko")?,
            decyduje: row.get("decyduje")?,
            do_kiedy: row.get("do_kiedy")?,
            zlozono: row.get("zlozono")?,
            tresc: row.get("tresc")?,
            dotyczy_tabela: row.get("dotyczy_tabela")?,
            dotyczy_id: row.get("dotyczy_id")?,
            dotyczy_skrot: row.get("dotyczy_skrot")?,
            stan: row.get("stan")?,
            rozstrzygniecie: row.get("rozstrzygniecie")?,
            warunek: row.get("warunek")?,
            uzasadnienie: row.get("uzasadnienie")?,
            podpisal: row.get("podpisal")?,
            za_kogo: row.get("za_kogo")?,
            podpis: row.get("podpis")?,
            podpisano: row.get("podpisano")?,
            zastepuje_decyzja_id: row.get("zastepuje_decyzja_id")?,
            powod_zbednosci: row.get("powod_zbednosci")?,
        })
    }
}

/// Prośby, które czekają na rozstrzygnięcie, najstarsze pierwsze; z nich Kierownik biura układa
/// listę dla każdego człowieka.
pub fn decyzje_czekajace(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<DecyzjeCzekajaceRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM sprawa_decyzje_wpisy
WHERE stan = 'czeka'
ORDER BY zlozono, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(DecyzjeCzekajaceRow::from_row).collect()
}

/// A row of `decyzje_zatwierdzone`.
#[derive(Debug, Clone, PartialEq)]
pub struct DecyzjeZatwierdzoneRow {
    pub id: String,
    pub rodzaj: WpisyRodzaj,
    pub prosi: String,
    pub prosi_stanowisko: WpisyProsiStanowisko,
    pub decyduje: String,
    pub do_kiedy: Option<String>,
    pub zlozono: String,
    pub tresc: Option<String>,
    pub dotyczy_tabela: Option<String>,
    pub dotyczy_id: Option<String>,
    pub dotyczy_skrot: Option<String>,
    pub stan: WpisyStan,
    pub rozstrzygniecie: Option<WpisyRozstrzygniecie>,
    pub warunek: Option<String>,
    pub uzasadnienie: Option<String>,
    pub podpisal: Option<String>,
    pub za_kogo: Option<String>,
    pub podpis: Option<String>,
    pub podpisano: Option<String>,
    pub zastepuje_decyzja_id: Option<String>,
    pub powod_zbednosci: Option<String>,
}

impl DecyzjeZatwierdzoneRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(DecyzjeZatwierdzoneRow {
            id: row.get("id")?,
            rodzaj: row.get("rodzaj")?,
            prosi: row.get("prosi")?,
            prosi_stanowisko: row.get("prosi_stanowisko")?,
            decyduje: row.get("decyduje")?,
            do_kiedy: row.get("do_kiedy")?,
            zlozono: row.get("zlozono")?,
            tresc: row.get("tresc")?,
            dotyczy_tabela: row.get("dotyczy_tabela")?,
            dotyczy_id: row.get("dotyczy_id")?,
            dotyczy_skrot: row.get("dotyczy_skrot")?,
            stan: row.get("stan")?,
            rozstrzygniecie: row.get("rozstrzygniecie")?,
            warunek: row.get("warunek")?,
            uzasadnienie: row.get("uzasadnienie")?,
            podpisal: row.get("podpisal")?,
            za_kogo: row.get("za_kogo")?,
            podpis: row.get("podpis")?,
            podpisano: row.get("podpisano")?,
            zastepuje_decyzja_id: row.get("zastepuje_decyzja_id")?,
            powod_zbednosci: row.get("powod_zbednosci")?,
        })
    }
}

/// Obowiązujące zgody: decyzje rozstrzygnięte zgodą albo zgodą z warunkiem, najnowsze pierwsze.
pub fn decyzje_zatwierdzone(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<DecyzjeZatwierdzoneRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM sprawa_decyzje_zatwierdzone
ORDER BY podpisano DESC, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(DecyzjeZatwierdzoneRow::from_row).collect()
}

/// The parameters of `decyzje_wiersza`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DecyzjeWierszaParams {
    pub dotyczy_tabela: String,
    pub dotyczy_id: String,
}

/// A row of `decyzje_wiersza`.
#[derive(Debug, Clone, PartialEq)]
pub struct DecyzjeWierszaRow {
    pub id: String,
    pub rodzaj: WpisyRodzaj,
    pub prosi: String,
    pub prosi_stanowisko: WpisyProsiStanowisko,
    pub decyduje: String,
    pub do_kiedy: Option<String>,
    pub zlozono: String,
    pub tresc: Option<String>,
    pub dotyczy_tabela: Option<String>,
    pub dotyczy_id: Option<String>,
    pub dotyczy_skrot: Option<String>,
    pub stan: WpisyStan,
    pub rozstrzygniecie: Option<WpisyRozstrzygniecie>,
    pub warunek: Option<String>,
    pub uzasadnienie: Option<String>,
    pub podpisal: Option<String>,
    pub za_kogo: Option<String>,
    pub podpis: Option<String>,
    pub podpisano: Option<String>,
    pub zastepuje_decyzja_id: Option<String>,
    pub powod_zbednosci: Option<String>,
}

impl DecyzjeWierszaRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(DecyzjeWierszaRow {
            id: row.get("id")?,
            rodzaj: row.get("rodzaj")?,
            prosi: row.get("prosi")?,
            prosi_stanowisko: row.get("prosi_stanowisko")?,
            decyduje: row.get("decyduje")?,
            do_kiedy: row.get("do_kiedy")?,
            zlozono: row.get("zlozono")?,
            tresc: row.get("tresc")?,
            dotyczy_tabela: row.get("dotyczy_tabela")?,
            dotyczy_id: row.get("dotyczy_id")?,
            dotyczy_skrot: row.get("dotyczy_skrot")?,
            stan: row.get("stan")?,
            rozstrzygniecie: row.get("rozstrzygniecie")?,
            warunek: row.get("warunek")?,
            uzasadnienie: row.get("uzasadnienie")?,
            podpisal: row.get("podpisal")?,
            za_kogo: row.get("za_kogo")?,
            podpis: row.get("podpis")?,
            podpisano: row.get("podpisano")?,
            zastepuje_decyzja_id: row.get("zastepuje_decyzja_id")?,
            powod_zbednosci: row.get("powod_zbednosci")?,
        })
    }
}

/// Decyzje, które wskazują jeden wiersz teczki, najnowsze pierwsze.
pub fn decyzje_wiersza(
    room: &impl xmeta_room::MetaRoom,
    params: &DecyzjeWierszaParams,
) -> Result<Vec<DecyzjeWierszaRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM sprawa_decyzje_wpisy
WHERE dotyczy_tabela = :dotyczy_tabela
  AND dotyczy_id = :dotyczy_id
ORDER BY zlozono DESC, id"#,
        parameters: Vec::from([
            (
                "dotyczy_tabela",
                xmeta_room::ToSqlValue::to_sql_value(&params.dotyczy_tabela),
            ),
            ("dotyczy_id", xmeta_room::ToSqlValue::to_sql_value(&params.dotyczy_id)),
        ]),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(DecyzjeWierszaRow::from_row).collect()
}

/// The parameters of `popros_o_decyzje`.
#[derive(Debug, Clone, PartialEq)]
pub struct PoprosODecyzjeParams {
    pub id: String,
    pub rodzaj: WpisyRodzaj,
    pub prosi: String,
    pub prosi_stanowisko: WpisyProsiStanowisko,
    pub decyduje: String,
    pub do_kiedy: Option<String>,
    pub zlozono: String,
    pub tresc: Option<String>,
    pub dotyczy_tabela: Option<String>,
    pub dotyczy_id: Option<String>,
    pub dotyczy_skrot: Option<String>,
    pub zastepuje_decyzja_id: Option<String>,
}

/// Prośba agenta o decyzję człowieka; identyfikator (UUID) nadaje aplikacja, żeby wiersz, który
/// czeka na decyzję, mógł go od razu wskazać.
pub fn popros_o_decyzje(
    room: &impl xmeta_room::MetaRoom,
    params: &PoprosODecyzjeParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO sprawa_decyzje_wpisy (id, rodzaj, prosi, prosi_stanowisko, decyduje, do_kiedy, zlozono,
  tresc, dotyczy_tabela, dotyczy_id, dotyczy_skrot, zastepuje_decyzja_id, stan)
VALUES (:id, :rodzaj, :prosi, :prosi_stanowisko, :decyduje, :do_kiedy, :zlozono, :tresc,
  :dotyczy_tabela, :dotyczy_id, :dotyczy_skrot, :zastepuje_decyzja_id, 'czeka')"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("rodzaj", xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj)),
                    ("prosi", xmeta_room::ToSqlValue::to_sql_value(&params.prosi)),
                    (
                        "prosi_stanowisko",
                        xmeta_room::ToSqlValue::to_sql_value(&params.prosi_stanowisko),
                    ),
                    ("decyduje", xmeta_room::ToSqlValue::to_sql_value(&params.decyduje)),
                    ("do_kiedy", xmeta_room::ToSqlValue::to_sql_value(&params.do_kiedy)),
                    ("zlozono", xmeta_room::ToSqlValue::to_sql_value(&params.zlozono)),
                    ("tresc", xmeta_room::ToSqlValue::to_sql_value(&params.tresc)),
                    (
                        "dotyczy_tabela",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dotyczy_tabela),
                    ),
                    (
                        "dotyczy_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dotyczy_id),
                    ),
                    (
                        "dotyczy_skrot",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dotyczy_skrot),
                    ),
                    (
                        "zastepuje_decyzja_id",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.zastepuje_decyzja_id,
                        ),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `rozstrzygnij_decyzje`.
#[derive(Debug, Clone, PartialEq)]
pub struct RozstrzygnijDecyzjeParams {
    pub rozstrzygniecie: WpisyRozstrzygniecie,
    pub warunek: Option<String>,
    pub uzasadnienie: Option<String>,
    pub podpisal: String,
    pub za_kogo: Option<String>,
    pub podpis: String,
    pub podpisano: String,
    pub id: String,
}

/// Rozstrzygnięcie prośby przez człowieka albo jego zastępcę, z podpisem kluczem xkeys.
pub fn rozstrzygnij_decyzje(
    room: &impl xmeta_room::MetaRoom,
    params: &RozstrzygnijDecyzjeParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE sprawa_decyzje_wpisy
SET stan = 'rozstrzygnieta',
  rozstrzygniecie = :rozstrzygniecie,
  warunek = :warunek,
  uzasadnienie = :uzasadnienie,
  podpisal = :podpisal,
  za_kogo = :za_kogo,
  podpis = :podpis,
  podpisano = :podpisano
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "rozstrzygniecie",
                        xmeta_room::ToSqlValue::to_sql_value(&params.rozstrzygniecie),
                    ),
                    ("warunek", xmeta_room::ToSqlValue::to_sql_value(&params.warunek)),
                    (
                        "uzasadnienie",
                        xmeta_room::ToSqlValue::to_sql_value(&params.uzasadnienie),
                    ),
                    ("podpisal", xmeta_room::ToSqlValue::to_sql_value(&params.podpisal)),
                    ("za_kogo", xmeta_room::ToSqlValue::to_sql_value(&params.za_kogo)),
                    ("podpis", xmeta_room::ToSqlValue::to_sql_value(&params.podpis)),
                    (
                        "podpisano",
                        xmeta_room::ToSqlValue::to_sql_value(&params.podpisano),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `uznaj_prosbe_za_zbedna`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct UznajProsbeZaZbednaParams {
    pub powod_zbednosci: String,
    pub id: String,
}

/// Prośba, która przestała być potrzebna, np. bo sprawa się zmieniła; zostaje z powodem.
pub fn uznaj_prosbe_za_zbedna(
    room: &impl xmeta_room::MetaRoom,
    params: &UznajProsbeZaZbednaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE sprawa_decyzje_wpisy
SET stan = 'zbedna',
  powod_zbednosci = :powod_zbednosci
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "powod_zbednosci",
                        xmeta_room::ToSqlValue::to_sql_value(&params.powod_zbednosci),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}
