//! The kind `najem-kaucja` of the room `cases/najem`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind: its name, the kinds it requires and its migrations in the order of their
/// numbers.
pub const KIND: xmeta_room::Kind = xmeta_room::Kind {
    name: "najem-kaucja",
    requires: &[
        &super::najem_czynsz::KIND,
        &super::najem_umowy::KIND,
        &super::sprawa_decyzje::KIND,
        &super::sprawa_dokumenty::KIND,
        &super::sprawa_korespondencja::KIND,
        &super::sprawa_metryka::KIND,
        &super::sprawa_rachunki::KIND,
        &super::sprawa_uczestnicy::KIND,
        &super::sprawa_wyplaty::KIND,
    ],
    migrations: &[
        r#"-- Kaucja najmu: pobranie, wykorzystanie na zaległość przy najmie instytucjonalnym i rozliczenie
-- po opróżnieniu lokalu z potrąceniami. Kwotę do zwrotu liczy program, a każde potrącenie
-- zatwierdza człowiek. Kwoty w groszach.
--
-- Kaucja zabezpiecza należności wynajmującego w dniu opróżnienia lokalu (art. 6 ust. 1 UOPL).
-- Zwrot to kaucja zwaloryzowana (czynsz z dnia zwrotu razy krotność z pobrania, nie mniej niż
-- kaucja pobrana, art. 6 ust. 3 UOPL) minus potrącenia, w ciągu miesiąca od opróżnienia lokalu
-- (art. 6 ust. 4, art. 19a ust. 5 i art. 19f ust. 6 UOPL); termin trzyma rdzeń. Gdy kaucja leży
-- u biura, zwrot jest wypłatą z rdzenia, która wskazuje zatwierdzone rozliczenie.

-- Pobranie kaucji: kwota, krotność czynszu i czynsz z dnia zawarcia, wpłata i miejsce, gdzie leży.
-- Rachunek biura nie jest rachunkiem uczestnika, więc kaucja u biura nie wskazuje rachunku z rdzenia.
CREATE TABLE najem_kaucja_pobrania (
  id TEXT PRIMARY KEY,
  najem_id TEXT NOT NULL REFERENCES najem_umowy_najmu (id),
  kwota INTEGER NOT NULL CHECK (kwota > 0),
  -- Krotność miesięcznego czynszu przyjęta przy pobraniu, np. 2 albo 1,5; z niej program liczy
  -- kaucję zwaloryzowaną.
  krotnosc REAL NOT NULL CHECK (krotnosc > 0),
  czynsz_przy_zawarciu INTEGER NOT NULL CHECK (czynsz_przy_zawarciu > 0),
  wplacono TEXT NOT NULL,
  lezy TEXT NOT NULL CHECK (lezy IN ('u_wynajmujacego', 'u_biura')),
  rachunek_id TEXT REFERENCES sprawa_rachunki_wpisy (id),

  CONSTRAINT rachunek_wynajmujacego_wskazany CHECK ((lezy = 'u_wynajmujacego') = (rachunek_id IS NOT NULL))
) STRICT;

-- Art. 19f ust. 7–8 UOPL: przy najmie instytucjonalnym właściciel pokrywa zaległość z kaucji,
-- a najemca uzupełnia ją w terminie z umowy albo z pisma.
CREATE TABLE najem_kaucja_wykorzystania (
  -- Klucz nadany przez bazę jest dodatni; zob. najem_kaucja_wykorzystania_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  pobranie_id TEXT NOT NULL REFERENCES najem_kaucja_pobrania (id),
  kwota INTEGER NOT NULL CHECK (kwota > 0),
  chwila TEXT NOT NULL,
  zaleglosc_id TEXT NOT NULL REFERENCES najem_czynsz_zaleglosci (id),
  uzupelnic_do TEXT,
  uzupelniono TEXT
) STRICT;

-- Rozliczenie kaucji po opróżnieniu lokalu: projekt z wyliczeniem programu, zatwierdzenie po zgodzie
-- człowieka na każde potrącenie i wysłanie stronom. Wypłata zwrotu wskazuje rozliczenie, więc
-- wiersz ma UUID.
CREATE TABLE najem_kaucja_rozliczenia (
  id TEXT PRIMARY KEY,
  pobranie_id TEXT NOT NULL REFERENCES najem_kaucja_pobrania (id),
  oproznienie TEXT NOT NULL,
  czynsz_w_dniu_zwrotu INTEGER NOT NULL CHECK (czynsz_w_dniu_zwrotu > 0),
  kwota_zwaloryzowana INTEGER NOT NULL CHECK (kwota_zwaloryzowana > 0),
  wyliczenie_sciezka TEXT NOT NULL,
  wyliczenie_skrot TEXT NOT NULL,
  stan TEXT NOT NULL CHECK (stan IN ('projekt', 'zatwierdzone', 'wyslane')),
  wysylka_id TEXT REFERENCES sprawa_korespondencja_wysylki (id),

  CONSTRAINT wyslane_ma_wysylke CHECK ((stan = 'wyslane') = (wysylka_id IS NOT NULL))
) STRICT;

-- Potrącenia z kaucji: należność wynajmującego z tytułem, kwotą i dokumentem, na którym stoi,
-- np. zaległy czynsz z salda albo naprawa z protokołu zwrotu i rachunku wykonawcy. Decyzja
-- wskazuje potrącenie, więc wiersz ma UUID.
CREATE TABLE najem_kaucja_potracenia (
  id TEXT PRIMARY KEY,
  rozliczenie_id TEXT NOT NULL REFERENCES najem_kaucja_rozliczenia (id),
  tytul TEXT NOT NULL CHECK (tytul <> ''),
  kwota INTEGER NOT NULL CHECK (kwota > 0),
  dokument_id TEXT NOT NULL REFERENCES sprawa_dokumenty_wpisy (id),
  decyzja_id TEXT REFERENCES sprawa_decyzje_wpisy (id)
) STRICT;

-- Pobranie ------------------------------------------------------------------------------------------

-- Pobrana kaucja mieści się w kaucji z umowy (a ta w granicy z ustawy), czynsz to czynsz z dnia
-- zawarcia, a kaucja wynajmującego leży na jego rachunku dopuszczonym decyzją (R4).
CREATE TRIGGER najem_kaucja_pobranie_w_granicy_umowy
BEFORE INSERT ON najem_kaucja_pobrania
WHEN NOT EXISTS (
  SELECT 1
  FROM najem_umowy_najmu
  WHERE id = NEW.najem_id
    AND NEW.kwota <= kaucja
    AND NEW.czynsz_przy_zawarciu = czynsz
)
BEGIN
  SELECT RAISE(ABORT, 'pobrana kaucja mieści się w kaucji z umowy najmu, a czynsz to czynsz z dnia zawarcia');
END;

CREATE TRIGGER najem_kaucja_pobranie_na_dopuszczony_rachunek
BEFORE INSERT ON najem_kaucja_pobrania
WHEN NEW.rachunek_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_rachunki_dopuszczone
    WHERE id = NEW.rachunek_id
  )
BEGIN
  SELECT RAISE(ABORT, 'kaucja idzie na rachunek dopuszczony decyzją i niezastąpiony (R4)');
END;

CREATE TRIGGER najem_kaucja_pobrania_bez_zmian
BEFORE UPDATE ON najem_kaucja_pobrania
BEGIN
  SELECT RAISE(ABORT, 'pobrania kaucji się nie zmienia');
END;

-- Rozliczenie kaucji pomija się tylko wtedy, gdy kaucji nie pobrano.
CREATE TRIGGER najem_kaucja_rozliczenie_przy_pobranej
BEFORE INSERT ON sprawa_metryka_etapy
WHEN NEW.etap = 'po_najmie'
  AND (
    SELECT etap = 'zwrot_lokalu'
    FROM sprawa_metryka_etapy
    ORDER BY id DESC
    LIMIT 1
  )
  AND EXISTS (
    SELECT 1
    FROM najem_kaucja_pobrania
  )
BEGIN
  SELECT RAISE(ABORT, 'po zwrocie lokalu z pobraną kaucją przychodzi rozliczenie kaucji');
END;

-- Wykorzystanie -------------------------------------------------------------------------------------

CREATE TRIGGER najem_kaucja_wykorzystanie_przy_instytucjonalnym
BEFORE INSERT ON najem_kaucja_wykorzystania
WHEN NEW.uzupelniono IS NOT NULL
  OR NOT EXISTS (
    SELECT 1
    FROM najem_kaucja_pobrania AS pobranie
    JOIN najem_umowy_najmu AS najem ON najem.id = pobranie.najem_id
    JOIN najem_czynsz_zaleglosci AS zaleglosc ON zaleglosc.najem_id = najem.id
    WHERE pobranie.id = NEW.pobranie_id
      AND zaleglosc.id = NEW.zaleglosc_id
      AND najem.rodzaj = 'instytucjonalny'
  )
BEGIN
  SELECT RAISE(ABORT, 'kaucję wykorzystuje się na zaległość tego najmu tylko przy najmie instytucjonalnym (art. 19f ust. 7 UOPL), jeszcze nieuzupełnioną');
END;

CREATE TRIGGER najem_kaucja_wykorzystanie_zmienia_tylko_uzupelnienie
BEFORE UPDATE ON najem_kaucja_wykorzystania
WHEN NEW.id IS NOT OLD.id
  OR NEW.pobranie_id IS NOT OLD.pobranie_id
  OR NEW.kwota IS NOT OLD.kwota
  OR NEW.chwila IS NOT OLD.chwila
  OR NEW.zaleglosc_id IS NOT OLD.zaleglosc_id
  OR (OLD.uzupelnic_do IS NOT NULL AND NEW.uzupelnic_do IS NOT OLD.uzupelnic_do)
  OR (OLD.uzupelniono IS NOT NULL AND NEW.uzupelniono IS NOT OLD.uzupelniono)
BEGIN
  SELECT RAISE(ABORT, 'wykorzystania kaucji się nie zmienia; termin i dzień uzupełnienia wpisuje się raz');
END;

-- Rozliczenie ---------------------------------------------------------------------------------------

-- Zatwierdzenie i wysłanie sprawdzają zmianę stanu, więc rozliczenie powstaje jako projekt.
CREATE TRIGGER najem_kaucja_rozliczenie_powstaje_jako_projekt
BEFORE INSERT ON najem_kaucja_rozliczenia
WHEN NEW.stan <> 'projekt'
BEGIN
  SELECT RAISE(ABORT, 'rozliczenie kaucji powstaje jako projekt');
END;

-- Art. 6 ust. 3 UOPL: kaucja zwaloryzowana nie jest mniejsza niż pobrana.
CREATE TRIGGER najem_kaucja_waloryzacja_nie_mniej_niz_pobrana
BEFORE INSERT ON najem_kaucja_rozliczenia
WHEN NEW.kwota_zwaloryzowana < (
  SELECT kwota
  FROM najem_kaucja_pobrania
  WHERE id = NEW.pobranie_id
)
BEGIN
  SELECT RAISE(ABORT, 'kaucja zwaloryzowana nie jest mniejsza niż pobrana (art. 6 ust. 3 UOPL)');
END;

CREATE TRIGGER najem_kaucja_rozliczenie_przejscia_stanu
BEFORE UPDATE OF stan ON najem_kaucja_rozliczenia
WHEN NEW.stan IS NOT OLD.stan
  AND NOT (
    (OLD.stan = 'projekt' AND NEW.stan = 'zatwierdzone')
    OR (OLD.stan = 'zatwierdzone' AND NEW.stan = 'wyslane')
  )
BEGIN
  SELECT RAISE(ABORT, 'rozliczenie kaucji przechodzi: projekt, zatwierdzone, wysłane');
END;

-- Każde potrącenie zatwierdza człowiek, a potrącenia nie przekraczają kaucji zwaloryzowanej.
CREATE TRIGGER najem_kaucja_zatwierdzenie_po_zgodzie_na_potracenia
BEFORE UPDATE OF stan ON najem_kaucja_rozliczenia
WHEN NEW.stan = 'zatwierdzone'
  AND OLD.stan = 'projekt'
  AND (
    EXISTS (
      SELECT 1
      FROM najem_kaucja_potracenia AS potracenie
      WHERE potracenie.rozliczenie_id = NEW.id
        AND NOT EXISTS (
          SELECT 1
          FROM sprawa_decyzje_zatwierdzone AS decyzja
          WHERE decyzja.id = potracenie.decyzja_id
            AND decyzja.rodzaj = 'inna'
            AND decyzja.dotyczy_tabela = 'najem_kaucja_potracenia'
            AND decyzja.dotyczy_id = potracenie.id
        )
    )
    OR coalesce(
      (
        SELECT sum(kwota)
        FROM najem_kaucja_potracenia
        WHERE rozliczenie_id = NEW.id
      ),
      0
    ) > NEW.kwota_zwaloryzowana
  )
BEGIN
  SELECT RAISE(ABORT, 'rozliczenie kaucji jest zatwierdzone, gdy każde potrącenie ma zatwierdzoną decyzję, a potrącenia nie przekraczają kaucji zwaloryzowanej');
END;

-- Wyliczenie i potrącenia są tym, na co zgodził się człowiek; po projekcie zmienia się tylko stan
-- i raz wpisana wysyłka do stron.
CREATE TRIGGER najem_kaucja_rozliczenie_po_projekcie_niezmienne
BEFORE UPDATE ON najem_kaucja_rozliczenia
WHEN OLD.stan <> 'projekt'
  AND (
    NEW.id IS NOT OLD.id
    OR NEW.pobranie_id IS NOT OLD.pobranie_id
    OR NEW.oproznienie IS NOT OLD.oproznienie
    OR NEW.czynsz_w_dniu_zwrotu IS NOT OLD.czynsz_w_dniu_zwrotu
    OR NEW.kwota_zwaloryzowana IS NOT OLD.kwota_zwaloryzowana
    OR NEW.wyliczenie_sciezka IS NOT OLD.wyliczenie_sciezka
    OR NEW.wyliczenie_skrot IS NOT OLD.wyliczenie_skrot
    OR (OLD.wysylka_id IS NOT NULL AND NEW.wysylka_id IS NOT OLD.wysylka_id)
  )
BEGIN
  SELECT RAISE(ABORT, 'zatwierdzonego rozliczenia kaucji się nie zmienia; zmienia się tylko stan i raz wpisana wysyłka');
END;

-- Waloryzację sprawdza się też przy poprawce projektu.
CREATE TRIGGER najem_kaucja_poprawiona_waloryzacja_nie_mniej_niz_pobrana
BEFORE UPDATE OF kwota_zwaloryzowana, pobranie_id ON najem_kaucja_rozliczenia
WHEN NEW.kwota_zwaloryzowana < (
  SELECT kwota
  FROM najem_kaucja_pobrania
  WHERE id = NEW.pobranie_id
)
BEGIN
  SELECT RAISE(ABORT, 'kaucja zwaloryzowana nie jest mniejsza niż pobrana (art. 6 ust. 3 UOPL)');
END;

-- Potrącenia ----------------------------------------------------------------------------------------

-- Potrącenie należy do projektu rozliczenia i powstaje bez zgody, którą dopisuje się później.
CREATE TRIGGER najem_kaucja_potracenie_w_projekcie
BEFORE INSERT ON najem_kaucja_potracenia
WHEN NEW.decyzja_id IS NOT NULL
  OR (
    SELECT stan <> 'projekt'
    FROM najem_kaucja_rozliczenia
    WHERE id = NEW.rozliczenie_id
  )
BEGIN
  SELECT RAISE(ABORT, 'potrącenie dopisuje się do projektu rozliczenia kaucji, bez zgody');
END;

CREATE TRIGGER najem_kaucja_potracenie_za_zgoda
BEFORE UPDATE OF decyzja_id ON najem_kaucja_potracenia
WHEN NEW.decyzja_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_decyzje_zatwierdzone AS decyzja
    WHERE decyzja.id = NEW.decyzja_id
      AND decyzja.rodzaj = 'inna'
      AND decyzja.dotyczy_tabela = 'najem_kaucja_potracenia'
      AND decyzja.dotyczy_id = NEW.id
  )
BEGIN
  SELECT RAISE(ABORT, 'potrącenie wskazuje zatwierdzoną decyzję inna, która je wskazuje');
END;

-- Zgoda dotyczy treści potrącenia; po zatwierdzeniu rozliczenia potrącenia stoją.
CREATE TRIGGER najem_kaucja_potracenie_bez_zmian
BEFORE UPDATE ON najem_kaucja_potracenia
WHEN NEW.id IS NOT OLD.id
  OR NEW.rozliczenie_id IS NOT OLD.rozliczenie_id
  OR NEW.tytul IS NOT OLD.tytul
  OR NEW.kwota IS NOT OLD.kwota
  OR NEW.dokument_id IS NOT OLD.dokument_id
  OR (OLD.decyzja_id IS NOT NULL AND NEW.decyzja_id IS NOT OLD.decyzja_id)
  OR (
    SELECT stan <> 'projekt'
    FROM najem_kaucja_rozliczenia
    WHERE id = OLD.rozliczenie_id
  )
BEGIN
  SELECT RAISE(ABORT, 'potrącenia się nie zmienia; zgodę wpisuje się raz, w projekcie rozliczenia');
END;

-- Zwrot kaucji --------------------------------------------------------------------------------------

-- Zwrot kaucji leżącej u biura to wypłata z rdzenia: wskazuje zatwierdzone (albo już wysłane)
-- rozliczenie, idzie do najemcy, ma kwotę zwaloryzowaną minus potrącenia i jest jedyną
-- nieanulowaną wypłatą tego rozliczenia.
CREATE TRIGGER najem_kaucja_zwrot_wedlug_rozliczenia
BEFORE INSERT ON sprawa_wyplaty_wpisy
WHEN NEW.podstawa_tabela = 'najem_kaucja_rozliczenia'
  AND (
    NOT EXISTS (
      SELECT 1
      FROM najem_kaucja_rozliczenia AS rozliczenie
      JOIN najem_kaucja_pobrania AS pobranie ON pobranie.id = rozliczenie.pobranie_id
      WHERE rozliczenie.id = NEW.podstawa_id
        AND rozliczenie.stan IN ('zatwierdzone', 'wyslane')
        AND pobranie.lezy = 'u_biura'
        AND NEW.kwota = rozliczenie.kwota_zwaloryzowana - coalesce(
          (
            SELECT sum(kwota)
            FROM najem_kaucja_potracenia
            WHERE rozliczenie_id = rozliczenie.id
          ),
          0
        )
    )
    OR NOT EXISTS (
      SELECT 1
      FROM sprawa_uczestnicy_role
      WHERE uczestnik_id = NEW.odbiorca_uczestnik_id
        AND rola = 'najemca'
        AND zakonczono IS NULL
    )
    OR EXISTS (
      SELECT 1
      FROM sprawa_wyplaty_wpisy
      WHERE podstawa_tabela = 'najem_kaucja_rozliczenia'
        AND podstawa_id = NEW.podstawa_id
        AND stan <> 'anulowana'
    )
  )
BEGIN
  SELECT RAISE(ABORT, 'zwrot kaucji u biura wskazuje zatwierdzone rozliczenie, idzie do najemcy w kwocie zwaloryzowanej minus potrącenia i jest jedyny');
END;

-- Bez usuwania i zastępowania ---------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER najem_kaucja_pobrania_bez_zastepowania
BEFORE INSERT ON najem_kaucja_pobrania
WHEN EXISTS (
  SELECT 1
  FROM najem_kaucja_pobrania
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER najem_kaucja_pobrania_bez_usuwania
BEFORE DELETE ON najem_kaucja_pobrania
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER najem_kaucja_wykorzystania_bez_zastepowania
BEFORE INSERT ON najem_kaucja_wykorzystania
WHEN EXISTS (
  SELECT 1
  FROM najem_kaucja_wykorzystania
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER najem_kaucja_wykorzystania_bez_usuwania
BEFORE DELETE ON najem_kaucja_wykorzystania
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER najem_kaucja_rozliczenia_bez_zastepowania
BEFORE INSERT ON najem_kaucja_rozliczenia
WHEN EXISTS (
  SELECT 1
  FROM najem_kaucja_rozliczenia
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER najem_kaucja_rozliczenia_bez_usuwania
BEFORE DELETE ON najem_kaucja_rozliczenia
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER najem_kaucja_potracenia_bez_zastepowania
BEFORE INSERT ON najem_kaucja_potracenia
WHEN EXISTS (
  SELECT 1
  FROM najem_kaucja_potracenia
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER najem_kaucja_potracenia_bez_usuwania
BEFORE DELETE ON najem_kaucja_potracenia
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;
"#,
    ],
};

/// The values of column `pobrania.lezy`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PobraniaLezy {
    UWynajmujacego,
    UBiura,
}

impl PobraniaLezy {
    /// Every value, in the order of the catalog.
    pub const ALL: &[PobraniaLezy] = &[
        PobraniaLezy::UWynajmujacego,
        PobraniaLezy::UBiura,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            PobraniaLezy::UWynajmujacego => "u_wynajmujacego",
            PobraniaLezy::UBiura => "u_biura",
        }
    }
}

impl xmeta_room::FromSqlValue for PobraniaLezy {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "u_wynajmujacego" => Ok(PobraniaLezy::UWynajmujacego),
                    "u_biura" => Ok(PobraniaLezy::UBiura),
                    other => Err(format!("{other} is no value of PobraniaLezy")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for PobraniaLezy {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `rozliczenia.stan`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RozliczeniaStan {
    Projekt,
    Zatwierdzone,
    Wyslane,
}

impl RozliczeniaStan {
    /// Every value, in the order of the catalog.
    pub const ALL: &[RozliczeniaStan] = &[
        RozliczeniaStan::Projekt,
        RozliczeniaStan::Zatwierdzone,
        RozliczeniaStan::Wyslane,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            RozliczeniaStan::Projekt => "projekt",
            RozliczeniaStan::Zatwierdzone => "zatwierdzone",
            RozliczeniaStan::Wyslane => "wyslane",
        }
    }
}

impl xmeta_room::FromSqlValue for RozliczeniaStan {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "projekt" => Ok(RozliczeniaStan::Projekt),
                    "zatwierdzone" => Ok(RozliczeniaStan::Zatwierdzone),
                    "wyslane" => Ok(RozliczeniaStan::Wyslane),
                    other => Err(format!("{other} is no value of RozliczeniaStan")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for RozliczeniaStan {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// A row of `pobrania_kaucji`.
#[derive(Debug, Clone, PartialEq)]
pub struct PobraniaKaucjiRow {
    pub id: String,
    pub najem_id: String,
    pub kwota: i64,
    pub krotnosc: f64,
    pub czynsz_przy_zawarciu: i64,
    pub wplacono: String,
    pub lezy: PobraniaLezy,
    pub rachunek_id: Option<String>,
}

impl PobraniaKaucjiRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(PobraniaKaucjiRow {
            id: row.get("id")?,
            najem_id: row.get("najem_id")?,
            kwota: row.get("kwota")?,
            krotnosc: row.get("krotnosc")?,
            czynsz_przy_zawarciu: row.get("czynsz_przy_zawarciu")?,
            wplacono: row.get("wplacono")?,
            lezy: row.get("lezy")?,
            rachunek_id: row.get("rachunek_id")?,
        })
    }
}

/// Pobrane kaucje z tym, gdzie leżą.
pub fn pobrania_kaucji(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<PobraniaKaucjiRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM najem_kaucja_pobrania
ORDER BY wplacono, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(PobraniaKaucjiRow::from_row).collect()
}

/// The parameters of `wpisz_pobranie_kaucji`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszPobranieKaucjiParams {
    pub id: String,
    pub najem_id: String,
    pub kwota: i64,
    pub krotnosc: f64,
    pub czynsz_przy_zawarciu: i64,
    pub wplacono: String,
    pub lezy: PobraniaLezy,
    pub rachunek_id: Option<String>,
}

/// Kaucja wpłacona przy zawarciu umowy; identyfikator (UUID) nadaje aplikacja.
pub fn wpisz_pobranie_kaucji(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszPobranieKaucjiParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO najem_kaucja_pobrania (id, najem_id, kwota, krotnosc, czynsz_przy_zawarciu, wplacono,
  lezy, rachunek_id)
VALUES (:id, :najem_id, :kwota, :krotnosc, :czynsz_przy_zawarciu, :wplacono, :lezy, :rachunek_id)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("najem_id", xmeta_room::ToSqlValue::to_sql_value(&params.najem_id)),
                    ("kwota", xmeta_room::ToSqlValue::to_sql_value(&params.kwota)),
                    ("krotnosc", xmeta_room::ToSqlValue::to_sql_value(&params.krotnosc)),
                    (
                        "czynsz_przy_zawarciu",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.czynsz_przy_zawarciu,
                        ),
                    ),
                    ("wplacono", xmeta_room::ToSqlValue::to_sql_value(&params.wplacono)),
                    ("lezy", xmeta_room::ToSqlValue::to_sql_value(&params.lezy)),
                    (
                        "rachunek_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.rachunek_id),
                    ),
                ]),
            },
        ],
    )
}

/// A row of `potracenia_z_kaucji`.
#[derive(Debug, Clone, PartialEq)]
pub struct PotraceniaZKaucjiRow {
    pub id: String,
    pub rozliczenie_id: String,
    pub tytul: String,
    pub kwota: i64,
    pub dokument_id: String,
    pub decyzja_id: Option<String>,
}

impl PotraceniaZKaucjiRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(PotraceniaZKaucjiRow {
            id: row.get("id")?,
            rozliczenie_id: row.get("rozliczenie_id")?,
            tytul: row.get("tytul")?,
            kwota: row.get("kwota")?,
            dokument_id: row.get("dokument_id")?,
            decyzja_id: row.get("decyzja_id")?,
        })
    }
}

/// Potrącenia z kaucji z dokumentem i zgodą.
pub fn potracenia_z_kaucji(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<PotraceniaZKaucjiRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM najem_kaucja_potracenia
ORDER BY rozliczenie_id, rowid"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(PotraceniaZKaucjiRow::from_row).collect()
}

/// The parameters of `dopisz_potracenie_z_kaucji`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DopiszPotracenieZKaucjiParams {
    pub id: String,
    pub rozliczenie_id: String,
    pub tytul: String,
    pub kwota: i64,
    pub dokument_id: String,
}

/// Należność wynajmującego do potrącenia, z dokumentem, na którym stoi; identyfikator (UUID)
/// nadaje aplikacja, bo wskazuje go decyzja.
pub fn dopisz_potracenie_z_kaucji(
    room: &impl xmeta_room::MetaRoom,
    params: &DopiszPotracenieZKaucjiParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO najem_kaucja_potracenia (id, rozliczenie_id, tytul, kwota, dokument_id)
VALUES (:id, :rozliczenie_id, :tytul, :kwota, :dokument_id)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    (
                        "rozliczenie_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.rozliczenie_id),
                    ),
                    ("tytul", xmeta_room::ToSqlValue::to_sql_value(&params.tytul)),
                    ("kwota", xmeta_room::ToSqlValue::to_sql_value(&params.kwota)),
                    (
                        "dokument_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dokument_id),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `wskaz_zgode_na_potracenie`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WskazZgodeNaPotracenieParams {
    pub decyzja_id: String,
    pub id: String,
}

/// Zatwierdzona decyzja inna o tym potrąceniu.
pub fn wskaz_zgode_na_potracenie(
    room: &impl xmeta_room::MetaRoom,
    params: &WskazZgodeNaPotracenieParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE najem_kaucja_potracenia
SET decyzja_id = :decyzja_id
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "decyzja_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.decyzja_id),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// A row of `rozliczenia_kaucji`.
#[derive(Debug, Clone, PartialEq)]
pub struct RozliczeniaKaucjiRow {
    pub id: String,
    pub pobranie_id: String,
    pub oproznienie: String,
    pub czynsz_w_dniu_zwrotu: i64,
    pub kwota_zwaloryzowana: i64,
    pub wyliczenie_sciezka: String,
    pub wyliczenie_skrot: String,
    pub stan: RozliczeniaStan,
    pub wysylka_id: Option<String>,
}

impl RozliczeniaKaucjiRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(RozliczeniaKaucjiRow {
            id: row.get("id")?,
            pobranie_id: row.get("pobranie_id")?,
            oproznienie: row.get("oproznienie")?,
            czynsz_w_dniu_zwrotu: row.get("czynsz_w_dniu_zwrotu")?,
            kwota_zwaloryzowana: row.get("kwota_zwaloryzowana")?,
            wyliczenie_sciezka: row.get("wyliczenie_sciezka")?,
            wyliczenie_skrot: row.get("wyliczenie_skrot")?,
            stan: row.get("stan")?,
            wysylka_id: row.get("wysylka_id")?,
        })
    }
}

/// Rozliczenia kaucji z kwotą zwaloryzowaną i stanem.
pub fn rozliczenia_kaucji(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<RozliczeniaKaucjiRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM najem_kaucja_rozliczenia
ORDER BY oproznienie, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(RozliczeniaKaucjiRow::from_row).collect()
}

/// The parameters of `przygotuj_rozliczenie_kaucji`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PrzygotujRozliczenieKaucjiParams {
    pub id: String,
    pub pobranie_id: String,
    pub oproznienie: String,
    pub czynsz_w_dniu_zwrotu: i64,
    pub kwota_zwaloryzowana: i64,
    pub wyliczenie_sciezka: String,
    pub wyliczenie_skrot: String,
}

/// Projekt rozliczenia z wyliczeniem programu po opróżnieniu lokalu; identyfikator (UUID) nadaje
/// aplikacja, bo wskazują go potrącenia i wypłata zwrotu.
pub fn przygotuj_rozliczenie_kaucji(
    room: &impl xmeta_room::MetaRoom,
    params: &PrzygotujRozliczenieKaucjiParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO najem_kaucja_rozliczenia (id, pobranie_id, oproznienie, czynsz_w_dniu_zwrotu,
  kwota_zwaloryzowana, wyliczenie_sciezka, wyliczenie_skrot, stan)
VALUES (:id, :pobranie_id, :oproznienie, :czynsz_w_dniu_zwrotu, :kwota_zwaloryzowana,
  :wyliczenie_sciezka, :wyliczenie_skrot, 'projekt')"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    (
                        "pobranie_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.pobranie_id),
                    ),
                    (
                        "oproznienie",
                        xmeta_room::ToSqlValue::to_sql_value(&params.oproznienie),
                    ),
                    (
                        "czynsz_w_dniu_zwrotu",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.czynsz_w_dniu_zwrotu,
                        ),
                    ),
                    (
                        "kwota_zwaloryzowana",
                        xmeta_room::ToSqlValue::to_sql_value(&params.kwota_zwaloryzowana),
                    ),
                    (
                        "wyliczenie_sciezka",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wyliczenie_sciezka),
                    ),
                    (
                        "wyliczenie_skrot",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wyliczenie_skrot),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `zatwierdz_rozliczenie_kaucji`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZatwierdzRozliczenieKaucjiParams {
    pub id: String,
}

/// Każde potrącenie ma zgodę człowieka; rozliczenie może iść do stron.
pub fn zatwierdz_rozliczenie_kaucji(
    room: &impl xmeta_room::MetaRoom,
    params: &ZatwierdzRozliczenieKaucjiParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE najem_kaucja_rozliczenia
SET stan = 'zatwierdzone'
WHERE id = :id"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_wyslanie_rozliczenia_kaucji`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujWyslanieRozliczeniaKaucjiParams {
    pub wysylka_id: String,
    pub id: String,
}

/// Rozliczenie wysłane stronom według wzoru.
pub fn odnotuj_wyslanie_rozliczenia_kaucji(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujWyslanieRozliczeniaKaucjiParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE najem_kaucja_rozliczenia
SET stan = 'wyslane',
  wysylka_id = :wysylka_id
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

/// A row of `wykorzystania_kaucji`.
#[derive(Debug, Clone, PartialEq)]
pub struct WykorzystaniaKaucjiRow {
    pub id: i64,
    pub pobranie_id: String,
    pub kwota: i64,
    pub chwila: String,
    pub zaleglosc_id: String,
    pub uzupelnic_do: Option<String>,
    pub uzupelniono: Option<String>,
}

impl WykorzystaniaKaucjiRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(WykorzystaniaKaucjiRow {
            id: row.get("id")?,
            pobranie_id: row.get("pobranie_id")?,
            kwota: row.get("kwota")?,
            chwila: row.get("chwila")?,
            zaleglosc_id: row.get("zaleglosc_id")?,
            uzupelnic_do: row.get("uzupelnic_do")?,
            uzupelniono: row.get("uzupelniono")?,
        })
    }
}

/// Pokrycia zaległości z kaucji przy najmie instytucjonalnym.
pub fn wykorzystania_kaucji(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<WykorzystaniaKaucjiRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM najem_kaucja_wykorzystania
ORDER BY chwila, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(WykorzystaniaKaucjiRow::from_row).collect()
}

/// The parameters of `wpisz_wykorzystanie_kaucji`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpiszWykorzystanieKaucjiParams {
    pub pobranie_id: String,
    pub kwota: i64,
    pub chwila: String,
    pub zaleglosc_id: String,
    pub uzupelnic_do: Option<String>,
}

/// Właściciel pokrył zaległość z kaucji; termin uzupełnienia z umowy albo z pisma.
pub fn wpisz_wykorzystanie_kaucji(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszWykorzystanieKaucjiParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO najem_kaucja_wykorzystania (pobranie_id, kwota, chwila, zaleglosc_id, uzupelnic_do)
VALUES (:pobranie_id, :kwota, :chwila, :zaleglosc_id, :uzupelnic_do)"#,
                parameters: Vec::from([
                    (
                        "pobranie_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.pobranie_id),
                    ),
                    ("kwota", xmeta_room::ToSqlValue::to_sql_value(&params.kwota)),
                    ("chwila", xmeta_room::ToSqlValue::to_sql_value(&params.chwila)),
                    (
                        "zaleglosc_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zaleglosc_id),
                    ),
                    (
                        "uzupelnic_do",
                        xmeta_room::ToSqlValue::to_sql_value(&params.uzupelnic_do),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_uzupelnienie_kaucji`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujUzupelnienieKaucjiParams {
    pub uzupelniono: String,
    pub id: i64,
}

/// Najemca uzupełnił kaucję.
pub fn odnotuj_uzupelnienie_kaucji(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujUzupelnienieKaucjiParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE najem_kaucja_wykorzystania
SET uzupelniono = :uzupelniono
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "uzupelniono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.uzupelniono),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}
