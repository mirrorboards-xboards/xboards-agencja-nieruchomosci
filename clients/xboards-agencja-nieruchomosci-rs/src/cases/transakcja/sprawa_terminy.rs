//! The kind `sprawa-terminy` of the room `cases/transakcja`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind: its name, the kinds it requires and its migrations in the order of their
/// numbers.
pub const KIND: xmeta_room::Kind = xmeta_room::Kind {
    name: "sprawa-terminy",
    requires: &[&super::sprawa_decyzje::KIND],
    migrations: &[
        r#"-- Terminy sprawy: ustawowe z podstawą prawną, z umów i wewnętrzne biura. Liczy je program,
-- a wpisuje i pilnuje agent sprawy, który ma termin w swoim profilu. Umówiony termin, np. akt,
-- ma wydarzenie w kalendarzu biura, a gdy profil wymaga zgody, wskazuje decyzję
-- potwierdzenie_terminu. Znacznik wątpliwości blokuje tę zgodę (trigger w sprawa-metryka).
--
-- Terminy z AML (np. informacja do GIIF) nie trafiają do teczki: są tajemnicą (art. 54 ust. 1 AML)
-- i pilnuje ich rejestr kontroli.
CREATE TABLE sprawa_terminy_wpisy (
  id TEXT PRIMARY KEY,
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('ustawowy', 'z_umowy', 'wewnetrzny')),
  co TEXT NOT NULL,
  -- Zdarzenie, od którego termin biegnie: wiersz teczki i jego chwila.
  zdarzenie_tabela TEXT,
  zdarzenie_id TEXT,
  zdarzenie_chwila TEXT,
  -- Dzień, a gdy trzeba, chwila końca; dni robocze liczy program.
  koniec TEXT NOT NULL,
  -- Przepis, postanowienie umowy albo standard działu.
  podstawa TEXT,
  pilnuje_stanowisko TEXT NOT NULL CHECK (pilnuje_stanowisko IN ('analityk_kredytowy', 'analityk_stanu_prawnego', 'kierownik_biura', 'kierownik_kontroli', 'kierownik_najmu', 'kierownik_posrednictwa', 'kierownik_transakcji', 'konsjerz', 'kontroler', 'koordynator_operacji', 'koordynator_transakcji', 'opiekun_najmu', 'opiekun_oferty', 'opiekun_poszukujacego', 'recepcja', 'zarzadca_obiektu')),
  stan TEXT NOT NULL CHECK (stan IN ('biegnie', 'dotrzymany', 'minal', 'zbedny')),
  -- Wydarzenie w kalendarzu biura, tekstem.
  kalendarz_odnosnik TEXT,
  potwierdzenie_decyzja_id TEXT REFERENCES sprawa_decyzje_wpisy (id),

  CONSTRAINT zdarzenie_tabela_razem_z_id CHECK ((zdarzenie_tabela IS NULL) = (zdarzenie_id IS NULL)),
  CONSTRAINT ustawowy_ma_podstawe_i_zdarzenie CHECK (
    rodzaj <> 'ustawowy' OR (podstawa IS NOT NULL AND zdarzenie_id IS NOT NULL AND zdarzenie_chwila IS NOT NULL)
  )
) STRICT;

-- Potwierdzenie sprawdza zmianę, więc termin nie może powstać od razu rozstrzygnięty.
CREATE TRIGGER sprawa_terminy_powstaje_biegnacy
BEFORE INSERT ON sprawa_terminy_wpisy
WHEN NEW.stan <> 'biegnie' OR NEW.potwierdzenie_decyzja_id IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'termin powstaje jako biegnący i niepotwierdzony');
END;

CREATE TRIGGER sprawa_terminy_potwierdzenie_wymaga_decyzji
BEFORE UPDATE OF potwierdzenie_decyzja_id ON sprawa_terminy_wpisy
WHEN NEW.potwierdzenie_decyzja_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_decyzje_zatwierdzone AS decyzja
    WHERE decyzja.id = NEW.potwierdzenie_decyzja_id
      AND decyzja.rodzaj = 'potwierdzenie_terminu'
      AND decyzja.dotyczy_tabela = 'sprawa_terminy_wpisy'
      AND decyzja.dotyczy_id = NEW.id
  )
BEGIN
  SELECT RAISE(ABORT, 'termin potwierdza tylko zatwierdzona decyzja potwierdzenie_terminu, która go wskazuje');
END;

-- Podpis decyzji potwierdzenie_terminu obejmuje skrót wiersza, więc potwierdzony termin zmienia
-- już tylko stan. Przesunięty termin to nowy wiersz z nowym potwierdzeniem, a stary staje się
-- zbędny. Potwierdzenia się nie zeruje ani nie podmienia.
CREATE TRIGGER sprawa_terminy_potwierdzony_niezmienny
BEFORE UPDATE ON sprawa_terminy_wpisy
WHEN OLD.potwierdzenie_decyzja_id IS NOT NULL
  AND (
    NEW.id IS NOT OLD.id
    OR NEW.rodzaj IS NOT OLD.rodzaj
    OR NEW.co IS NOT OLD.co
    OR NEW.zdarzenie_tabela IS NOT OLD.zdarzenie_tabela
    OR NEW.zdarzenie_id IS NOT OLD.zdarzenie_id
    OR NEW.zdarzenie_chwila IS NOT OLD.zdarzenie_chwila
    OR NEW.koniec IS NOT OLD.koniec
    OR NEW.podstawa IS NOT OLD.podstawa
    OR NEW.pilnuje_stanowisko IS NOT OLD.pilnuje_stanowisko
    OR NEW.kalendarz_odnosnik IS NOT OLD.kalendarz_odnosnik
    OR NEW.potwierdzenie_decyzja_id IS NOT OLD.potwierdzenie_decyzja_id
  )
BEGIN
  SELECT RAISE(ABORT, 'potwierdzonego terminu się nie zmienia; zmienia się tylko jego stan, a przesunięty termin to nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER sprawa_terminy_wpisy_bez_zastepowania
BEFORE INSERT ON sprawa_terminy_wpisy
WHEN EXISTS (
  SELECT 1
  FROM sprawa_terminy_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER sprawa_terminy_wpisy_bez_usuwania
BEFORE DELETE ON sprawa_terminy_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;
"#,
    ],
};

/// The values of column `wpisy.rodzaj`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyRodzaj {
    Ustawowy,
    ZUmowy,
    Wewnetrzny,
}

impl WpisyRodzaj {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyRodzaj] = &[
        WpisyRodzaj::Ustawowy,
        WpisyRodzaj::ZUmowy,
        WpisyRodzaj::Wewnetrzny,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyRodzaj::Ustawowy => "ustawowy",
            WpisyRodzaj::ZUmowy => "z_umowy",
            WpisyRodzaj::Wewnetrzny => "wewnetrzny",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyRodzaj {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "ustawowy" => Ok(WpisyRodzaj::Ustawowy),
                    "z_umowy" => Ok(WpisyRodzaj::ZUmowy),
                    "wewnetrzny" => Ok(WpisyRodzaj::Wewnetrzny),
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

/// The values of column `wpisy.pilnuje_stanowisko`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyPilnujeStanowisko {
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

impl WpisyPilnujeStanowisko {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyPilnujeStanowisko] = &[
        WpisyPilnujeStanowisko::AnalitykKredytowy,
        WpisyPilnujeStanowisko::AnalitykStanuPrawnego,
        WpisyPilnujeStanowisko::KierownikBiura,
        WpisyPilnujeStanowisko::KierownikKontroli,
        WpisyPilnujeStanowisko::KierownikNajmu,
        WpisyPilnujeStanowisko::KierownikPosrednictwa,
        WpisyPilnujeStanowisko::KierownikTransakcji,
        WpisyPilnujeStanowisko::Konsjerz,
        WpisyPilnujeStanowisko::Kontroler,
        WpisyPilnujeStanowisko::KoordynatorOperacji,
        WpisyPilnujeStanowisko::KoordynatorTransakcji,
        WpisyPilnujeStanowisko::OpiekunNajmu,
        WpisyPilnujeStanowisko::OpiekunOferty,
        WpisyPilnujeStanowisko::OpiekunPoszukujacego,
        WpisyPilnujeStanowisko::Recepcja,
        WpisyPilnujeStanowisko::ZarzadcaObiektu,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyPilnujeStanowisko::AnalitykKredytowy => "analityk_kredytowy",
            WpisyPilnujeStanowisko::AnalitykStanuPrawnego => "analityk_stanu_prawnego",
            WpisyPilnujeStanowisko::KierownikBiura => "kierownik_biura",
            WpisyPilnujeStanowisko::KierownikKontroli => "kierownik_kontroli",
            WpisyPilnujeStanowisko::KierownikNajmu => "kierownik_najmu",
            WpisyPilnujeStanowisko::KierownikPosrednictwa => "kierownik_posrednictwa",
            WpisyPilnujeStanowisko::KierownikTransakcji => "kierownik_transakcji",
            WpisyPilnujeStanowisko::Konsjerz => "konsjerz",
            WpisyPilnujeStanowisko::Kontroler => "kontroler",
            WpisyPilnujeStanowisko::KoordynatorOperacji => "koordynator_operacji",
            WpisyPilnujeStanowisko::KoordynatorTransakcji => "koordynator_transakcji",
            WpisyPilnujeStanowisko::OpiekunNajmu => "opiekun_najmu",
            WpisyPilnujeStanowisko::OpiekunOferty => "opiekun_oferty",
            WpisyPilnujeStanowisko::OpiekunPoszukujacego => "opiekun_poszukujacego",
            WpisyPilnujeStanowisko::Recepcja => "recepcja",
            WpisyPilnujeStanowisko::ZarzadcaObiektu => "zarzadca_obiektu",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyPilnujeStanowisko {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "analityk_kredytowy" => Ok(WpisyPilnujeStanowisko::AnalitykKredytowy),
                    "analityk_stanu_prawnego" => {
                        Ok(WpisyPilnujeStanowisko::AnalitykStanuPrawnego)
                    }
                    "kierownik_biura" => Ok(WpisyPilnujeStanowisko::KierownikBiura),
                    "kierownik_kontroli" => Ok(WpisyPilnujeStanowisko::KierownikKontroli),
                    "kierownik_najmu" => Ok(WpisyPilnujeStanowisko::KierownikNajmu),
                    "kierownik_posrednictwa" => {
                        Ok(WpisyPilnujeStanowisko::KierownikPosrednictwa)
                    }
                    "kierownik_transakcji" => {
                        Ok(WpisyPilnujeStanowisko::KierownikTransakcji)
                    }
                    "konsjerz" => Ok(WpisyPilnujeStanowisko::Konsjerz),
                    "kontroler" => Ok(WpisyPilnujeStanowisko::Kontroler),
                    "koordynator_operacji" => {
                        Ok(WpisyPilnujeStanowisko::KoordynatorOperacji)
                    }
                    "koordynator_transakcji" => {
                        Ok(WpisyPilnujeStanowisko::KoordynatorTransakcji)
                    }
                    "opiekun_najmu" => Ok(WpisyPilnujeStanowisko::OpiekunNajmu),
                    "opiekun_oferty" => Ok(WpisyPilnujeStanowisko::OpiekunOferty),
                    "opiekun_poszukujacego" => {
                        Ok(WpisyPilnujeStanowisko::OpiekunPoszukujacego)
                    }
                    "recepcja" => Ok(WpisyPilnujeStanowisko::Recepcja),
                    "zarzadca_obiektu" => Ok(WpisyPilnujeStanowisko::ZarzadcaObiektu),
                    other => {
                        Err(format!("{other} is no value of WpisyPilnujeStanowisko"))
                    }
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyPilnujeStanowisko {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wpisy.stan`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyStan {
    Biegnie,
    Dotrzymany,
    Minal,
    Zbedny,
}

impl WpisyStan {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyStan] = &[
        WpisyStan::Biegnie,
        WpisyStan::Dotrzymany,
        WpisyStan::Minal,
        WpisyStan::Zbedny,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyStan::Biegnie => "biegnie",
            WpisyStan::Dotrzymany => "dotrzymany",
            WpisyStan::Minal => "minal",
            WpisyStan::Zbedny => "zbedny",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyStan {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "biegnie" => Ok(WpisyStan::Biegnie),
                    "dotrzymany" => Ok(WpisyStan::Dotrzymany),
                    "minal" => Ok(WpisyStan::Minal),
                    "zbedny" => Ok(WpisyStan::Zbedny),
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

/// A row of `terminy`.
#[derive(Debug, Clone, PartialEq)]
pub struct TerminyRow {
    pub id: String,
    pub rodzaj: WpisyRodzaj,
    pub co: String,
    pub zdarzenie_tabela: Option<String>,
    pub zdarzenie_id: Option<String>,
    pub zdarzenie_chwila: Option<String>,
    pub koniec: String,
    pub podstawa: Option<String>,
    pub pilnuje_stanowisko: WpisyPilnujeStanowisko,
    pub stan: WpisyStan,
    pub kalendarz_odnosnik: Option<String>,
    pub potwierdzenie_decyzja_id: Option<String>,
}

impl TerminyRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(TerminyRow {
            id: row.get("id")?,
            rodzaj: row.get("rodzaj")?,
            co: row.get("co")?,
            zdarzenie_tabela: row.get("zdarzenie_tabela")?,
            zdarzenie_id: row.get("zdarzenie_id")?,
            zdarzenie_chwila: row.get("zdarzenie_chwila")?,
            koniec: row.get("koniec")?,
            podstawa: row.get("podstawa")?,
            pilnuje_stanowisko: row.get("pilnuje_stanowisko")?,
            stan: row.get("stan")?,
            kalendarz_odnosnik: row.get("kalendarz_odnosnik")?,
            potwierdzenie_decyzja_id: row.get("potwierdzenie_decyzja_id")?,
        })
    }
}

/// Terminy sprawy, najbliższy pierwszy.
pub fn terminy(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<TerminyRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM sprawa_terminy_wpisy
ORDER BY koniec, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(TerminyRow::from_row).collect()
}

/// A row of `terminy_biegnace`.
#[derive(Debug, Clone, PartialEq)]
pub struct TerminyBiegnaceRow {
    pub id: String,
    pub rodzaj: WpisyRodzaj,
    pub co: String,
    pub zdarzenie_tabela: Option<String>,
    pub zdarzenie_id: Option<String>,
    pub zdarzenie_chwila: Option<String>,
    pub koniec: String,
    pub podstawa: Option<String>,
    pub pilnuje_stanowisko: WpisyPilnujeStanowisko,
    pub stan: WpisyStan,
    pub kalendarz_odnosnik: Option<String>,
    pub potwierdzenie_decyzja_id: Option<String>,
}

impl TerminyBiegnaceRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(TerminyBiegnaceRow {
            id: row.get("id")?,
            rodzaj: row.get("rodzaj")?,
            co: row.get("co")?,
            zdarzenie_tabela: row.get("zdarzenie_tabela")?,
            zdarzenie_id: row.get("zdarzenie_id")?,
            zdarzenie_chwila: row.get("zdarzenie_chwila")?,
            koniec: row.get("koniec")?,
            podstawa: row.get("podstawa")?,
            pilnuje_stanowisko: row.get("pilnuje_stanowisko")?,
            stan: row.get("stan")?,
            kalendarz_odnosnik: row.get("kalendarz_odnosnik")?,
            potwierdzenie_decyzja_id: row.get("potwierdzenie_decyzja_id")?,
        })
    }
}

/// Terminy, które jeszcze biegną, najbliższy pierwszy.
pub fn terminy_biegnace(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<TerminyBiegnaceRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM sprawa_terminy_wpisy
WHERE stan = 'biegnie'
ORDER BY koniec, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(TerminyBiegnaceRow::from_row).collect()
}

/// The parameters of `wpisz_termin`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszTerminParams {
    pub id: String,
    pub rodzaj: WpisyRodzaj,
    pub co: String,
    pub zdarzenie_tabela: Option<String>,
    pub zdarzenie_id: Option<String>,
    pub zdarzenie_chwila: Option<String>,
    pub koniec: String,
    pub podstawa: Option<String>,
    pub pilnuje_stanowisko: WpisyPilnujeStanowisko,
    pub kalendarz_odnosnik: Option<String>,
}

/// Termin wyliczony programem albo umówiony; biegnie od zdarzenia.
pub fn wpisz_termin(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszTerminParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO sprawa_terminy_wpisy (id, rodzaj, co, zdarzenie_tabela, zdarzenie_id, zdarzenie_chwila,
  koniec, podstawa, pilnuje_stanowisko, kalendarz_odnosnik, stan)
VALUES (:id, :rodzaj, :co, :zdarzenie_tabela, :zdarzenie_id, :zdarzenie_chwila, :koniec, :podstawa,
  :pilnuje_stanowisko, :kalendarz_odnosnik, 'biegnie')"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("rodzaj", xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj)),
                    ("co", xmeta_room::ToSqlValue::to_sql_value(&params.co)),
                    (
                        "zdarzenie_tabela",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zdarzenie_tabela),
                    ),
                    (
                        "zdarzenie_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zdarzenie_id),
                    ),
                    (
                        "zdarzenie_chwila",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zdarzenie_chwila),
                    ),
                    ("koniec", xmeta_room::ToSqlValue::to_sql_value(&params.koniec)),
                    ("podstawa", xmeta_room::ToSqlValue::to_sql_value(&params.podstawa)),
                    (
                        "pilnuje_stanowisko",
                        xmeta_room::ToSqlValue::to_sql_value(&params.pilnuje_stanowisko),
                    ),
                    (
                        "kalendarz_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.kalendarz_odnosnik),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `przelicz_termin`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PrzeliczTerminParams {
    pub zdarzenie_tabela: Option<String>,
    pub zdarzenie_id: Option<String>,
    pub zdarzenie_chwila: Option<String>,
    pub koniec: String,
    pub kalendarz_odnosnik: Option<String>,
    pub id: String,
}

/// Nowy koniec niepotwierdzonego terminu, np. po zmianie zdarzenia. Potwierdzonego terminu się nie
/// przelicza: przesunięty akt to nowy termin z nowym potwierdzeniem, a stary staje się zbędny.
pub fn przelicz_termin(
    room: &impl xmeta_room::MetaRoom,
    params: &PrzeliczTerminParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE sprawa_terminy_wpisy
SET zdarzenie_tabela = :zdarzenie_tabela,
  zdarzenie_id = :zdarzenie_id,
  zdarzenie_chwila = :zdarzenie_chwila,
  koniec = :koniec,
  kalendarz_odnosnik = :kalendarz_odnosnik
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "zdarzenie_tabela",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zdarzenie_tabela),
                    ),
                    (
                        "zdarzenie_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zdarzenie_id),
                    ),
                    (
                        "zdarzenie_chwila",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zdarzenie_chwila),
                    ),
                    ("koniec", xmeta_room::ToSqlValue::to_sql_value(&params.koniec)),
                    (
                        "kalendarz_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.kalendarz_odnosnik),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `zmien_stan_terminu`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZmienStanTerminuParams {
    pub stan: WpisyStan,
    pub id: String,
}

/// Termin dotrzymany, który minął albo przestał być potrzebny; minięty trafia do raportu agenta.
pub fn zmien_stan_terminu(
    room: &impl xmeta_room::MetaRoom,
    params: &ZmienStanTerminuParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE sprawa_terminy_wpisy
SET stan = :stan
WHERE id = :id"#,
                parameters: Vec::from([
                    ("stan", xmeta_room::ToSqlValue::to_sql_value(&params.stan)),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_potwierdzenie_terminu`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujPotwierdzenieTerminuParams {
    pub potwierdzenie_decyzja_id: String,
    pub id: String,
}

/// Umówiony termin potwierdzony zatwierdzoną decyzją potwierdzenie_terminu.
pub fn odnotuj_potwierdzenie_terminu(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujPotwierdzenieTerminuParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE sprawa_terminy_wpisy
SET potwierdzenie_decyzja_id = :potwierdzenie_decyzja_id
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "potwierdzenie_decyzja_id",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.potwierdzenie_decyzja_id,
                        ),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}
