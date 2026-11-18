//! The kind `poszukiwanie-propozycje` of the room `cases/poszukiwanie`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind's name.
pub const KIND: &str = "poszukiwanie-propozycje";

/// The migrations of the kind, in the order of their numbers.
pub const MIGRATIONS: &[&str] = &[
    r#"-- Propozycje ofert dla klienta, prezentacje i pytania otwarte do strony, która sprzedaje albo
-- wynajmuje. Pisze Opiekun poszukującego. Kwoty w groszach.
--
-- Propozycja to oferta z portalu, z katalogu ofert biura albo od innego biura, porównana
-- z bieżącą potwierdzoną wersją profilu. Dane z ogłoszenia i z katalogu są twierdzeniem strony,
-- która sprzedaje albo wynajmuje. Ofertę z katalogu teczka wskazuje tekstem (inny pokój): wpis
-- i datę jego wersji. Ofertę niepubliczną Opiekun pokazuje tylko klientowi w granicach zgody
-- z katalogu; granice porównuje aplikacja. Przedstawienie propozycji klientowi to praca według
-- umowy pośrednictwa, więc nie jest wcześniejsze niż sprawa_umowy_praca_od.
--
-- Wynik przeglądania księgi Opiekun zapisuje bez oceny skutków; ocenia je Analityk stanu prawnego
-- w transakcji. Danych klienta druga strona dostaje tylko tyle, ile wymaga umówienie prezentacji;
-- szerszy zakres wymaga zatwierdzonej decyzji „inna”, która wskazuje prezentację.

CREATE TABLE poszukiwanie_propozycje_wpisy (
  id TEXT PRIMARY KEY,
  -- Wersja profilu, z którą Opiekun porównał ofertę.
  profil_wersja_id TEXT NOT NULL REFERENCES poszukiwanie_profile_wersje (id),

  -- Skąd oferta: kopia ogłoszenia z portalem i numerem, którą widział klient, albo wpis
  -- z katalogu ofert biura z datą wersji, i kto ją prowadzi.
  zrodlo TEXT NOT NULL CHECK (zrodlo IN ('portal', 'katalog_biura', 'inne_biuro')),
  ogloszenie_sciezka TEXT,
  ogloszenie_skrot TEXT,
  portal TEXT,
  numer TEXT,
  katalog_odnosnik TEXT,
  katalog_wersja_od TEXT,
  prowadzi TEXT NOT NULL CHECK (prowadzi IN ('wlasciciel', 'inne_biuro', 'opiekun_oferty')),
  niepubliczna INTEGER NOT NULL CHECK (niepubliczna IN (0, 1)),

  -- Ocena: warunki spełnione, życzenia niespełnione i rozbieżności zauważone jako fakt, np.
  -- ogłoszenie wróciło pod nowym numerem z innym metrażem.
  zgodnosc TEXT NOT NULL,
  niespelnione_zyczenia TEXT,
  rozbieznosci TEXT,

  -- Księga i cena: wynik przeglądania księgi, gdy Opiekun zna numer, cena z ogłoszenia i ceny
  -- podobnych transakcji z rejestru cen nieruchomości (art. 181a pkt 6 ugn) jako dokument.
  numer_kw TEXT CHECK (numer_kw GLOB '[A-Z][A-Z][0-9][A-Z]/[0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9]/[0-9]'),
  kw_przegladano TEXT,
  kw_ogloszajacy_wpisany TEXT CHECK (kw_ogloszajacy_wpisany IN ('tak', 'nie', 'nieznane')),
  kw_wpisy_iii_iv TEXT,
  cena_ogloszenia INTEGER NOT NULL CHECK (cena_ogloszenia > 0),
  ceny_rcn_dokument_id TEXT REFERENCES sprawa_dokumenty_wpisy (id),
  utworzono TEXT NOT NULL,

  -- Klient: kiedy dostał propozycję i co odpowiedział.
  przedstawiono TEXT,
  reakcja TEXT CHECK (reakcja IN ('chce_obejrzec', 'odrzuca', 'czeka')),
  reakcja_powod TEXT,

  CONSTRAINT z_portalu_z_kopia_i_numerem CHECK (
    zrodlo <> 'portal' OR (portal IS NOT NULL AND numer IS NOT NULL AND ogloszenie_sciezka IS NOT NULL)
  ),
  CONSTRAINT kopia_ze_skrotem CHECK ((ogloszenie_sciezka IS NULL) = (ogloszenie_skrot IS NULL)),
  CONSTRAINT z_katalogu_z_odnosnikiem_i_wersja CHECK (
    (zrodlo = 'katalog_biura') = (katalog_odnosnik IS NOT NULL)
    AND (zrodlo = 'katalog_biura') = (katalog_wersja_od IS NOT NULL)
  ),
  -- Ofertę biura prowadzi Opiekun oferty i tylko ona bywa niepubliczna.
  CONSTRAINT z_katalogu_prowadzi_opiekun_oferty CHECK ((zrodlo = 'katalog_biura') = (prowadzi = 'opiekun_oferty')),
  CONSTRAINT niepubliczna_tylko_z_katalogu CHECK (niepubliczna = 0 OR zrodlo = 'katalog_biura'),
  CONSTRAINT przegladanie_ksiegi_z_numerem_i_wynikiem CHECK (
    (kw_przegladano IS NULL) = (kw_ogloszajacy_wpisany IS NULL)
    AND (kw_przegladano IS NULL OR numer_kw IS NOT NULL)
    AND (kw_wpisy_iii_iv IS NULL OR kw_przegladano IS NOT NULL)
  ),
  CONSTRAINT przedstawiono_po_utworzeniu CHECK (przedstawiono IS NULL OR przedstawiono >= utworzono),
  CONSTRAINT reakcja_po_przedstawieniu CHECK (reakcja IS NULL OR przedstawiono IS NOT NULL),
  CONSTRAINT powod_przy_reakcji CHECK (reakcja_powod IS NULL OR reakcja IS NOT NULL)
) STRICT;

-- Prezentacja oglądanej nieruchomości z opinią klienta i notatką pośrednika.
CREATE TABLE poszukiwanie_propozycje_prezentacje (
  id TEXT PRIMARY KEY,
  propozycja_id TEXT NOT NULL REFERENCES poszukiwanie_propozycje_wpisy (id),
  termin TEXT NOT NULL,
  kto_pokazuje TEXT NOT NULL CHECK (kto_pokazuje IN ('wlasciciel', 'inne_biuro', 'posrednik_biura')),
  -- Dane klienta podane drugiej stronie, np. „imię, nazwisko, telefon”.
  dane_klienta_podane TEXT NOT NULL,
  szerszy_zakres INTEGER NOT NULL CHECK (szerszy_zakres IN (0, 1)),
  szerszy_zakres_decyzja_id TEXT REFERENCES sprawa_decyzje_wpisy (id),
  -- Kto przyjdzie po stronie klienta i czy przyjdzie pośrednik.
  uczestnicy TEXT,
  opinia_klienta TEXT,
  notatka_posrednika TEXT,
  odwolano TEXT,
  odwolal TEXT,
  powod_odwolania TEXT,

  CONSTRAINT szerszy_zakres_z_decyzja CHECK ((szerszy_zakres = 1) = (szerszy_zakres_decyzja_id IS NOT NULL)),
  CONSTRAINT odwolanie_kto_kiedy_dlaczego CHECK (
    (odwolano IS NULL) = (odwolal IS NULL)
    AND (odwolano IS NULL) = (powod_odwolania IS NULL)
  ),
  CONSTRAINT odwolana_bez_opinii_i_notatki CHECK (
    odwolano IS NULL OR (opinia_klienta IS NULL AND notatka_posrednika IS NULL)
  )
) STRICT;

-- Pytania klienta do strony, która pokazuje nieruchomość, i jej odpowiedzi, gdy przyjdą.
CREATE TABLE poszukiwanie_propozycje_pytania (
  -- Klucz nadany przez bazę jest dodatni; zob. poszukiwanie_propozycje_pytania_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  prezentacja_id TEXT NOT NULL REFERENCES poszukiwanie_propozycje_prezentacje (id),
  pytanie TEXT NOT NULL,
  -- Odpowiedź jest twierdzeniem drugiej strony z wpływem, w którym przyszła.
  odpowiedz TEXT,
  odpowiedz_wplyw_id TEXT REFERENCES sprawa_korespondencja_wplywy (id),

  CONSTRAINT odpowiedz_z_wplywem CHECK ((odpowiedz IS NULL) = (odpowiedz_wplyw_id IS NULL))
) STRICT;

-- Propozycję porównuje się z bieżącą wersją profilu, którą klient potwierdził.
CREATE TRIGGER poszukiwanie_propozycje_z_biezacym_profilem
BEFORE INSERT ON poszukiwanie_propozycje_wpisy
WHEN NOT EXISTS (
  SELECT 1
  FROM poszukiwanie_profile_biezace
  WHERE id = NEW.profil_wersja_id
)
BEGIN
  SELECT RAISE(ABORT, 'propozycja wskazuje bieżącą wersję profilu potwierdzoną przez klienta');
END;

-- Przedstawienie i reakcja klienta sprawdzają zmianę, więc propozycja powstaje
-- nieprzedstawiona.
CREATE TRIGGER poszukiwanie_propozycje_powstaje_nieprzedstawiona
BEFORE INSERT ON poszukiwanie_propozycje_wpisy
WHEN NEW.przedstawiono IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'propozycja powstaje nieprzedstawiona');
END;

-- Ceny podobnych transakcji to dokument z plikiem.
CREATE TRIGGER poszukiwanie_propozycje_ceny_rcn_z_plikiem
BEFORE INSERT ON poszukiwanie_propozycje_wpisy
WHEN NEW.ceny_rcn_dokument_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_dokumenty_wpisy
    WHERE id = NEW.ceny_rcn_dokument_id
      AND stan IN ('otrzymany', 'sprawdzony')
  )
BEGIN
  SELECT RAISE(ABORT, 'ceny transakcji z rejestru cen to dokument z plikiem: otrzymany albo sprawdzony');
END;

CREATE TRIGGER poszukiwanie_propozycje_zmienione_ceny_rcn_z_plikiem
BEFORE UPDATE OF ceny_rcn_dokument_id ON poszukiwanie_propozycje_wpisy
WHEN NEW.ceny_rcn_dokument_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_dokumenty_wpisy
    WHERE id = NEW.ceny_rcn_dokument_id
      AND stan IN ('otrzymany', 'sprawdzony')
  )
BEGIN
  SELECT RAISE(ABORT, 'ceny transakcji z rejestru cen to dokument z plikiem: otrzymany albo sprawdzony');
END;

-- Przedstawienie propozycji to praca według zawartej umowy pośrednictwa (art. 15 ust. 3
-- i art. 21 ust. 2 upk).
CREATE TRIGGER poszukiwanie_propozycje_przedstawienie_od_pracy
BEFORE UPDATE OF przedstawiono ON poszukiwanie_propozycje_wpisy
WHEN NEW.przedstawiono IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_umowy_wpisy AS umowa
    JOIN sprawa_umowy_praca_od AS praca ON praca.umowa_id = umowa.id
    WHERE umowa.rodzaj = 'posrednictwo'
      AND umowa.stan = 'zawarta'
      AND NEW.przedstawiono >= praca.od
  )
BEGIN
  SELECT RAISE(ABORT, 'propozycję przedstawia się przy zawartej umowie pośrednictwa, nie wcześniej niż praca od');
END;

-- Źródła i oceny propozycji się nie zmienia; przedstawienie, wynik przeglądania księgi, ceny
-- transakcji i rozbieżności wpisuje się raz, a reakcję zmienia tylko klient, który czekał.
CREATE TRIGGER poszukiwanie_propozycje_wpisy_bez_zmian
BEFORE UPDATE ON poszukiwanie_propozycje_wpisy
WHEN NEW.id IS NOT OLD.id
  OR NEW.profil_wersja_id IS NOT OLD.profil_wersja_id
  OR NEW.zrodlo IS NOT OLD.zrodlo
  OR NEW.ogloszenie_sciezka IS NOT OLD.ogloszenie_sciezka
  OR NEW.ogloszenie_skrot IS NOT OLD.ogloszenie_skrot
  OR NEW.portal IS NOT OLD.portal
  OR NEW.numer IS NOT OLD.numer
  OR NEW.katalog_odnosnik IS NOT OLD.katalog_odnosnik
  OR NEW.katalog_wersja_od IS NOT OLD.katalog_wersja_od
  OR NEW.prowadzi IS NOT OLD.prowadzi
  OR NEW.niepubliczna IS NOT OLD.niepubliczna
  OR NEW.zgodnosc IS NOT OLD.zgodnosc
  OR NEW.niespelnione_zyczenia IS NOT OLD.niespelnione_zyczenia
  OR NEW.cena_ogloszenia IS NOT OLD.cena_ogloszenia
  OR NEW.utworzono IS NOT OLD.utworzono
  OR (OLD.rozbieznosci IS NOT NULL AND NEW.rozbieznosci IS NOT OLD.rozbieznosci)
  OR (OLD.numer_kw IS NOT NULL AND NEW.numer_kw IS NOT OLD.numer_kw)
  OR (OLD.kw_przegladano IS NOT NULL AND (
    NEW.kw_przegladano IS NOT OLD.kw_przegladano
    OR NEW.kw_ogloszajacy_wpisany IS NOT OLD.kw_ogloszajacy_wpisany
    OR NEW.kw_wpisy_iii_iv IS NOT OLD.kw_wpisy_iii_iv
  ))
  OR (OLD.ceny_rcn_dokument_id IS NOT NULL AND NEW.ceny_rcn_dokument_id IS NOT OLD.ceny_rcn_dokument_id)
  OR (OLD.przedstawiono IS NOT NULL AND NEW.przedstawiono IS NOT OLD.przedstawiono)
  OR (OLD.reakcja IN ('chce_obejrzec', 'odrzuca') AND (
    NEW.reakcja IS NOT OLD.reakcja
    OR NEW.reakcja_powod IS NOT OLD.reakcja_powod
  ))
BEGIN
  SELECT RAISE(ABORT, 'propozycji się nie zmienia; przedstawienie, księgę, ceny i rozbieżności wpisuje się raz, a reakcja zmienia się tylko po czekaniu');
END;

-- Prezentacja dotyczy propozycji, którą klient dostał.
CREATE TRIGGER poszukiwanie_propozycje_prezentacja_przedstawionej
BEFORE INSERT ON poszukiwanie_propozycje_prezentacje
WHEN NOT EXISTS (
  SELECT 1
  FROM poszukiwanie_propozycje_wpisy
  WHERE id = NEW.propozycja_id
    AND przedstawiono IS NOT NULL
)
BEGIN
  SELECT RAISE(ABORT, 'prezentacja dotyczy propozycji przedstawionej klientowi');
END;

-- Szerszy zakres danych klienta dla drugiej strony stoi na zatwierdzonej decyzji „inna”, która
-- wskazuje prezentację.
CREATE TRIGGER poszukiwanie_propozycje_szerszy_zakres_za_zgoda
BEFORE INSERT ON poszukiwanie_propozycje_prezentacje
WHEN NEW.szerszy_zakres = 1
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_decyzje_zatwierdzone AS decyzja
    WHERE decyzja.id = NEW.szerszy_zakres_decyzja_id
      AND decyzja.rodzaj = 'inna'
      AND decyzja.dotyczy_tabela = 'poszukiwanie_propozycje_prezentacje'
      AND decyzja.dotyczy_id = NEW.id
  )
BEGIN
  SELECT RAISE(ABORT, 'szerszy zakres danych klienta wymaga zatwierdzonej decyzji inna, która wskazuje prezentację');
END;

-- Prezentacja powstaje umówiona; opinię, notatkę albo odwołanie wpisuje się potem raz. Inny
-- termin to odwołanie tej prezentacji i nowa prezentacja.
CREATE TRIGGER poszukiwanie_propozycje_prezentacja_powstaje_umowiona
BEFORE INSERT ON poszukiwanie_propozycje_prezentacje
WHEN NEW.opinia_klienta IS NOT NULL
  OR NEW.notatka_posrednika IS NOT NULL
  OR NEW.odwolano IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'prezentacja powstaje umówiona, bez opinii, notatki i odwołania');
END;

CREATE TRIGGER poszukiwanie_propozycje_prezentacje_bez_zmian
BEFORE UPDATE ON poszukiwanie_propozycje_prezentacje
WHEN OLD.odwolano IS NOT NULL
  OR NEW.id IS NOT OLD.id
  OR NEW.propozycja_id IS NOT OLD.propozycja_id
  OR NEW.termin IS NOT OLD.termin
  OR NEW.kto_pokazuje IS NOT OLD.kto_pokazuje
  OR NEW.dane_klienta_podane IS NOT OLD.dane_klienta_podane
  OR NEW.szerszy_zakres IS NOT OLD.szerszy_zakres
  OR NEW.szerszy_zakres_decyzja_id IS NOT OLD.szerszy_zakres_decyzja_id
  OR NEW.uczestnicy IS NOT OLD.uczestnicy
  OR (OLD.opinia_klienta IS NOT NULL AND NEW.opinia_klienta IS NOT OLD.opinia_klienta)
  OR (OLD.notatka_posrednika IS NOT NULL AND NEW.notatka_posrednika IS NOT OLD.notatka_posrednika)
BEGIN
  SELECT RAISE(ABORT, 'prezentacji się nie zmienia; opinię, notatkę albo odwołanie wpisuje się raz');
END;

-- Pytanie powstaje bez odpowiedzi; odpowiedź wpisuje się raz, gdy przyjdzie.
CREATE TRIGGER poszukiwanie_propozycje_pytanie_powstaje_otwarte
BEFORE INSERT ON poszukiwanie_propozycje_pytania
WHEN NEW.odpowiedz IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'pytanie powstaje otwarte; odpowiedź wpisuje się, gdy przyjdzie');
END;

CREATE TRIGGER poszukiwanie_propozycje_pytania_bez_zmian
BEFORE UPDATE ON poszukiwanie_propozycje_pytania
WHEN NEW.id IS NOT OLD.id
  OR NEW.prezentacja_id IS NOT OLD.prezentacja_id
  OR NEW.pytanie IS NOT OLD.pytanie
  OR (OLD.odpowiedz IS NOT NULL AND (
    NEW.odpowiedz IS NOT OLD.odpowiedz
    OR NEW.odpowiedz_wplyw_id IS NOT OLD.odpowiedz_wplyw_id
  ))
BEGIN
  SELECT RAISE(ABORT, 'pytania się nie zmienia; odpowiedź wpisuje się raz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER poszukiwanie_propozycje_wpisy_bez_zastepowania
BEFORE INSERT ON poszukiwanie_propozycje_wpisy
WHEN EXISTS (
  SELECT 1
  FROM poszukiwanie_propozycje_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER poszukiwanie_propozycje_wpisy_bez_usuwania
BEFORE DELETE ON poszukiwanie_propozycje_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER poszukiwanie_propozycje_prezentacje_bez_zastepowania
BEFORE INSERT ON poszukiwanie_propozycje_prezentacje
WHEN EXISTS (
  SELECT 1
  FROM poszukiwanie_propozycje_prezentacje
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER poszukiwanie_propozycje_prezentacje_bez_usuwania
BEFORE DELETE ON poszukiwanie_propozycje_prezentacje
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER poszukiwanie_propozycje_pytania_bez_zastepowania
BEFORE INSERT ON poszukiwanie_propozycje_pytania
WHEN EXISTS (
  SELECT 1
  FROM poszukiwanie_propozycje_pytania
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER poszukiwanie_propozycje_pytania_bez_usuwania
BEFORE DELETE ON poszukiwanie_propozycje_pytania
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;
"#,
];

/// The values of column `prezentacje.kto_pokazuje`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrezentacjeKtoPokazuje {
    Wlasciciel,
    InneBiuro,
    PosrednikBiura,
}

impl PrezentacjeKtoPokazuje {
    /// Every value, in the order of the catalog.
    pub const ALL: &[PrezentacjeKtoPokazuje] = &[
        PrezentacjeKtoPokazuje::Wlasciciel,
        PrezentacjeKtoPokazuje::InneBiuro,
        PrezentacjeKtoPokazuje::PosrednikBiura,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            PrezentacjeKtoPokazuje::Wlasciciel => "wlasciciel",
            PrezentacjeKtoPokazuje::InneBiuro => "inne_biuro",
            PrezentacjeKtoPokazuje::PosrednikBiura => "posrednik_biura",
        }
    }
}

impl xmeta_room::FromSqlValue for PrezentacjeKtoPokazuje {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "wlasciciel" => Ok(PrezentacjeKtoPokazuje::Wlasciciel),
                    "inne_biuro" => Ok(PrezentacjeKtoPokazuje::InneBiuro),
                    "posrednik_biura" => Ok(PrezentacjeKtoPokazuje::PosrednikBiura),
                    other => {
                        Err(format!("{other} is no value of PrezentacjeKtoPokazuje"))
                    }
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for PrezentacjeKtoPokazuje {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wpisy.zrodlo`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyZrodlo {
    Portal,
    KatalogBiura,
    InneBiuro,
}

impl WpisyZrodlo {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyZrodlo] = &[
        WpisyZrodlo::Portal,
        WpisyZrodlo::KatalogBiura,
        WpisyZrodlo::InneBiuro,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyZrodlo::Portal => "portal",
            WpisyZrodlo::KatalogBiura => "katalog_biura",
            WpisyZrodlo::InneBiuro => "inne_biuro",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyZrodlo {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "portal" => Ok(WpisyZrodlo::Portal),
                    "katalog_biura" => Ok(WpisyZrodlo::KatalogBiura),
                    "inne_biuro" => Ok(WpisyZrodlo::InneBiuro),
                    other => Err(format!("{other} is no value of WpisyZrodlo")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyZrodlo {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wpisy.prowadzi`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyProwadzi {
    Wlasciciel,
    InneBiuro,
    OpiekunOferty,
}

impl WpisyProwadzi {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyProwadzi] = &[
        WpisyProwadzi::Wlasciciel,
        WpisyProwadzi::InneBiuro,
        WpisyProwadzi::OpiekunOferty,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyProwadzi::Wlasciciel => "wlasciciel",
            WpisyProwadzi::InneBiuro => "inne_biuro",
            WpisyProwadzi::OpiekunOferty => "opiekun_oferty",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyProwadzi {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "wlasciciel" => Ok(WpisyProwadzi::Wlasciciel),
                    "inne_biuro" => Ok(WpisyProwadzi::InneBiuro),
                    "opiekun_oferty" => Ok(WpisyProwadzi::OpiekunOferty),
                    other => Err(format!("{other} is no value of WpisyProwadzi")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyProwadzi {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wpisy.kw_ogloszajacy_wpisany`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyKwOgloszajacyWpisany {
    Tak,
    Nie,
    Nieznane,
}

impl WpisyKwOgloszajacyWpisany {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyKwOgloszajacyWpisany] = &[
        WpisyKwOgloszajacyWpisany::Tak,
        WpisyKwOgloszajacyWpisany::Nie,
        WpisyKwOgloszajacyWpisany::Nieznane,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyKwOgloszajacyWpisany::Tak => "tak",
            WpisyKwOgloszajacyWpisany::Nie => "nie",
            WpisyKwOgloszajacyWpisany::Nieznane => "nieznane",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyKwOgloszajacyWpisany {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "tak" => Ok(WpisyKwOgloszajacyWpisany::Tak),
                    "nie" => Ok(WpisyKwOgloszajacyWpisany::Nie),
                    "nieznane" => Ok(WpisyKwOgloszajacyWpisany::Nieznane),
                    other => {
                        Err(format!("{other} is no value of WpisyKwOgloszajacyWpisany"))
                    }
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyKwOgloszajacyWpisany {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wpisy.reakcja`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyReakcja {
    ChceObejrzec,
    Odrzuca,
    Czeka,
}

impl WpisyReakcja {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyReakcja] = &[
        WpisyReakcja::ChceObejrzec,
        WpisyReakcja::Odrzuca,
        WpisyReakcja::Czeka,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyReakcja::ChceObejrzec => "chce_obejrzec",
            WpisyReakcja::Odrzuca => "odrzuca",
            WpisyReakcja::Czeka => "czeka",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyReakcja {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "chce_obejrzec" => Ok(WpisyReakcja::ChceObejrzec),
                    "odrzuca" => Ok(WpisyReakcja::Odrzuca),
                    "czeka" => Ok(WpisyReakcja::Czeka),
                    other => Err(format!("{other} is no value of WpisyReakcja")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyReakcja {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// A row of `prezentacje_propozycji`.
#[derive(Debug, Clone, PartialEq)]
pub struct PrezentacjePropozycjiRow {
    pub id: String,
    pub propozycja_id: String,
    pub termin: String,
    pub kto_pokazuje: PrezentacjeKtoPokazuje,
    pub dane_klienta_podane: String,
    pub szerszy_zakres: bool,
    pub szerszy_zakres_decyzja_id: Option<String>,
    pub uczestnicy: Option<String>,
    pub opinia_klienta: Option<String>,
    pub notatka_posrednika: Option<String>,
    pub odwolano: Option<String>,
    pub odwolal: Option<String>,
    pub powod_odwolania: Option<String>,
}

impl PrezentacjePropozycjiRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(PrezentacjePropozycjiRow {
            id: row.get("id")?,
            propozycja_id: row.get("propozycja_id")?,
            termin: row.get("termin")?,
            kto_pokazuje: row.get("kto_pokazuje")?,
            dane_klienta_podane: row.get("dane_klienta_podane")?,
            szerszy_zakres: row.get("szerszy_zakres")?,
            szerszy_zakres_decyzja_id: row.get("szerszy_zakres_decyzja_id")?,
            uczestnicy: row.get("uczestnicy")?,
            opinia_klienta: row.get("opinia_klienta")?,
            notatka_posrednika: row.get("notatka_posrednika")?,
            odwolano: row.get("odwolano")?,
            odwolal: row.get("odwolal")?,
            powod_odwolania: row.get("powod_odwolania")?,
        })
    }
}

/// Prezentacje oglądanych nieruchomości w kolejności terminów.
pub fn prezentacje_propozycji(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<PrezentacjePropozycjiRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM poszukiwanie_propozycje_prezentacje
ORDER BY termin, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(PrezentacjePropozycjiRow::from_row).collect()
}

/// A row of `pytania_otwarte`.
#[derive(Debug, Clone, PartialEq)]
pub struct PytaniaOtwarteRow {
    pub id: i64,
    pub prezentacja_id: String,
    pub pytanie: String,
    pub odpowiedz: Option<String>,
    pub odpowiedz_wplyw_id: Option<String>,
}

impl PytaniaOtwarteRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(PytaniaOtwarteRow {
            id: row.get("id")?,
            prezentacja_id: row.get("prezentacja_id")?,
            pytanie: row.get("pytanie")?,
            odpowiedz: row.get("odpowiedz")?,
            odpowiedz_wplyw_id: row.get("odpowiedz_wplyw_id")?,
        })
    }
}

/// Pytania klienta do strony, która pokazuje nieruchomość, bez odpowiedzi.
pub fn pytania_otwarte(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<PytaniaOtwarteRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM poszukiwanie_propozycje_pytania
WHERE odpowiedz IS NULL
ORDER BY prezentacja_id, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(PytaniaOtwarteRow::from_row).collect()
}

/// The parameters of `umow_prezentacje_propozycji`.
#[derive(Debug, Clone, PartialEq)]
pub struct UmowPrezentacjePropozycjiParams {
    pub id: String,
    pub propozycja_id: String,
    pub termin: String,
    pub kto_pokazuje: PrezentacjeKtoPokazuje,
    pub dane_klienta_podane: String,
    pub szerszy_zakres: bool,
    pub szerszy_zakres_decyzja_id: Option<String>,
    pub uczestnicy: Option<String>,
}

/// Prezentacja propozycji, którą klient dostał, z danymi klienta podanymi drugiej stronie;
/// szerszy zakres wskazuje zatwierdzoną decyzję człowieka. Identyfikator (UUID) nadaje aplikacja,
/// bo wskazują go pytania i decyzja.
pub fn umow_prezentacje_propozycji(
    room: &impl xmeta_room::MetaRoom,
    params: &UmowPrezentacjePropozycjiParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO poszukiwanie_propozycje_prezentacje (id, propozycja_id, termin, kto_pokazuje,
  dane_klienta_podane, szerszy_zakres, szerszy_zakres_decyzja_id, uczestnicy)
VALUES (:id, :propozycja_id, :termin, :kto_pokazuje, :dane_klienta_podane, :szerszy_zakres,
  :szerszy_zakres_decyzja_id, :uczestnicy)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    (
                        "propozycja_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.propozycja_id),
                    ),
                    ("termin", xmeta_room::ToSqlValue::to_sql_value(&params.termin)),
                    (
                        "kto_pokazuje",
                        xmeta_room::ToSqlValue::to_sql_value(&params.kto_pokazuje),
                    ),
                    (
                        "dane_klienta_podane",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dane_klienta_podane),
                    ),
                    (
                        "szerszy_zakres",
                        xmeta_room::ToSqlValue::to_sql_value(&params.szerszy_zakres),
                    ),
                    (
                        "szerszy_zakres_decyzja_id",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.szerszy_zakres_decyzja_id,
                        ),
                    ),
                    (
                        "uczestnicy",
                        xmeta_room::ToSqlValue::to_sql_value(&params.uczestnicy),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_opinie_klienta`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujOpinieKlientaParams {
    pub opinia_klienta: String,
    pub id: String,
}

/// Co klientowi się podobało, co nie i czy chce złożyć ofertę; wpisuje się ją raz.
pub fn odnotuj_opinie_klienta(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujOpinieKlientaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE poszukiwanie_propozycje_prezentacje
SET opinia_klienta = :opinia_klienta
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "opinia_klienta",
                        xmeta_room::ToSqlValue::to_sql_value(&params.opinia_klienta),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_notatke_posrednika`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujNotatkePosrednikaParams {
    pub notatka_posrednika: String,
    pub id: String,
}

/// Co pośrednik zauważył na miejscu; wpisuje się ją raz.
pub fn odnotuj_notatke_posrednika(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujNotatkePosrednikaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE poszukiwanie_propozycje_prezentacje
SET notatka_posrednika = :notatka_posrednika
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "notatka_posrednika",
                        xmeta_room::ToSqlValue::to_sql_value(&params.notatka_posrednika),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odwolaj_prezentacje_propozycji`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdwolajPrezentacjePropozycjiParams {
    pub odwolano: String,
    pub odwolal: String,
    pub powod_odwolania: String,
    pub id: String,
}

/// Kto, kiedy i dlaczego odwołał prezentację.
pub fn odwolaj_prezentacje_propozycji(
    room: &impl xmeta_room::MetaRoom,
    params: &OdwolajPrezentacjePropozycjiParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE poszukiwanie_propozycje_prezentacje
SET odwolano = :odwolano,
  odwolal = :odwolal,
  powod_odwolania = :powod_odwolania
WHERE id = :id"#,
                parameters: Vec::from([
                    ("odwolano", xmeta_room::ToSqlValue::to_sql_value(&params.odwolano)),
                    ("odwolal", xmeta_room::ToSqlValue::to_sql_value(&params.odwolal)),
                    (
                        "powod_odwolania",
                        xmeta_room::ToSqlValue::to_sql_value(&params.powod_odwolania),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `dopisz_pytanie_do_prezentacji`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DopiszPytanieDoPrezentacjiParams {
    pub prezentacja_id: String,
    pub pytanie: String,
}

/// Pytanie, o które klient chce dopytać drugą stronę.
pub fn dopisz_pytanie_do_prezentacji(
    room: &impl xmeta_room::MetaRoom,
    params: &DopiszPytanieDoPrezentacjiParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO poszukiwanie_propozycje_pytania (prezentacja_id, pytanie)
VALUES (:prezentacja_id, :pytanie)"#,
                parameters: Vec::from([
                    (
                        "prezentacja_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.prezentacja_id),
                    ),
                    ("pytanie", xmeta_room::ToSqlValue::to_sql_value(&params.pytanie)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_odpowiedz_na_pytanie`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujOdpowiedzNaPytanieParams {
    pub odpowiedz: String,
    pub odpowiedz_wplyw_id: String,
    pub id: i64,
}

/// Odpowiedź drugiej strony z wpływem, w którym przyszła; wpisuje się ją raz.
pub fn odnotuj_odpowiedz_na_pytanie(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujOdpowiedzNaPytanieParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE poszukiwanie_propozycje_pytania
SET odpowiedz = :odpowiedz,
  odpowiedz_wplyw_id = :odpowiedz_wplyw_id
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "odpowiedz",
                        xmeta_room::ToSqlValue::to_sql_value(&params.odpowiedz),
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

/// A row of `propozycje`.
#[derive(Debug, Clone, PartialEq)]
pub struct PropozycjeRow {
    pub id: String,
    pub profil_wersja_id: String,
    pub zrodlo: WpisyZrodlo,
    pub ogloszenie_sciezka: Option<String>,
    pub ogloszenie_skrot: Option<String>,
    pub portal: Option<String>,
    pub numer: Option<String>,
    pub katalog_odnosnik: Option<String>,
    pub katalog_wersja_od: Option<String>,
    pub prowadzi: WpisyProwadzi,
    pub niepubliczna: bool,
    pub zgodnosc: String,
    pub niespelnione_zyczenia: Option<String>,
    pub rozbieznosci: Option<String>,
    pub numer_kw: Option<String>,
    pub kw_przegladano: Option<String>,
    pub kw_ogloszajacy_wpisany: Option<WpisyKwOgloszajacyWpisany>,
    pub kw_wpisy_iii_iv: Option<String>,
    pub cena_ogloszenia: i64,
    pub ceny_rcn_dokument_id: Option<String>,
    pub utworzono: String,
    pub przedstawiono: Option<String>,
    pub reakcja: Option<WpisyReakcja>,
    pub reakcja_powod: Option<String>,
}

impl PropozycjeRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(PropozycjeRow {
            id: row.get("id")?,
            profil_wersja_id: row.get("profil_wersja_id")?,
            zrodlo: row.get("zrodlo")?,
            ogloszenie_sciezka: row.get("ogloszenie_sciezka")?,
            ogloszenie_skrot: row.get("ogloszenie_skrot")?,
            portal: row.get("portal")?,
            numer: row.get("numer")?,
            katalog_odnosnik: row.get("katalog_odnosnik")?,
            katalog_wersja_od: row.get("katalog_wersja_od")?,
            prowadzi: row.get("prowadzi")?,
            niepubliczna: row.get("niepubliczna")?,
            zgodnosc: row.get("zgodnosc")?,
            niespelnione_zyczenia: row.get("niespelnione_zyczenia")?,
            rozbieznosci: row.get("rozbieznosci")?,
            numer_kw: row.get("numer_kw")?,
            kw_przegladano: row.get("kw_przegladano")?,
            kw_ogloszajacy_wpisany: row.get("kw_ogloszajacy_wpisany")?,
            kw_wpisy_iii_iv: row.get("kw_wpisy_iii_iv")?,
            cena_ogloszenia: row.get("cena_ogloszenia")?,
            ceny_rcn_dokument_id: row.get("ceny_rcn_dokument_id")?,
            utworzono: row.get("utworzono")?,
            przedstawiono: row.get("przedstawiono")?,
            reakcja: row.get("reakcja")?,
            reakcja_powod: row.get("reakcja_powod")?,
        })
    }
}

/// Propozycje ofert dla klienta, najnowsze pierwsze.
pub fn propozycje(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<PropozycjeRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM poszukiwanie_propozycje_wpisy
ORDER BY utworzono DESC, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(PropozycjeRow::from_row).collect()
}

/// The parameters of `wpisz_propozycje`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszPropozycjeParams {
    pub id: String,
    pub profil_wersja_id: String,
    pub zrodlo: WpisyZrodlo,
    pub ogloszenie_sciezka: Option<String>,
    pub ogloszenie_skrot: Option<String>,
    pub portal: Option<String>,
    pub numer: Option<String>,
    pub katalog_odnosnik: Option<String>,
    pub katalog_wersja_od: Option<String>,
    pub prowadzi: WpisyProwadzi,
    pub niepubliczna: bool,
    pub zgodnosc: String,
    pub niespelnione_zyczenia: Option<String>,
    pub rozbieznosci: Option<String>,
    pub numer_kw: Option<String>,
    pub cena_ogloszenia: i64,
    pub ceny_rcn_dokument_id: Option<String>,
    pub utworzono: String,
}

/// Oferta z portalu, z katalogu ofert biura albo od innego biura, porównana z bieżącą potwierdzoną
/// wersją profilu; identyfikator (UUID) nadaje aplikacja, bo wskazują go prezentacje i oferty
/// klienta.
pub fn wpisz_propozycje(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszPropozycjeParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO poszukiwanie_propozycje_wpisy (id, profil_wersja_id, zrodlo, ogloszenie_sciezka,
  ogloszenie_skrot, portal, numer, katalog_odnosnik, katalog_wersja_od, prowadzi, niepubliczna,
  zgodnosc, niespelnione_zyczenia, rozbieznosci, numer_kw, cena_ogloszenia, ceny_rcn_dokument_id,
  utworzono)
VALUES (:id, :profil_wersja_id, :zrodlo, :ogloszenie_sciezka, :ogloszenie_skrot, :portal, :numer,
  :katalog_odnosnik, :katalog_wersja_od, :prowadzi, :niepubliczna, :zgodnosc,
  :niespelnione_zyczenia, :rozbieznosci, :numer_kw, :cena_ogloszenia, :ceny_rcn_dokument_id,
  :utworzono)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    (
                        "profil_wersja_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.profil_wersja_id),
                    ),
                    ("zrodlo", xmeta_room::ToSqlValue::to_sql_value(&params.zrodlo)),
                    (
                        "ogloszenie_sciezka",
                        xmeta_room::ToSqlValue::to_sql_value(&params.ogloszenie_sciezka),
                    ),
                    (
                        "ogloszenie_skrot",
                        xmeta_room::ToSqlValue::to_sql_value(&params.ogloszenie_skrot),
                    ),
                    ("portal", xmeta_room::ToSqlValue::to_sql_value(&params.portal)),
                    ("numer", xmeta_room::ToSqlValue::to_sql_value(&params.numer)),
                    (
                        "katalog_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.katalog_odnosnik),
                    ),
                    (
                        "katalog_wersja_od",
                        xmeta_room::ToSqlValue::to_sql_value(&params.katalog_wersja_od),
                    ),
                    ("prowadzi", xmeta_room::ToSqlValue::to_sql_value(&params.prowadzi)),
                    (
                        "niepubliczna",
                        xmeta_room::ToSqlValue::to_sql_value(&params.niepubliczna),
                    ),
                    ("zgodnosc", xmeta_room::ToSqlValue::to_sql_value(&params.zgodnosc)),
                    (
                        "niespelnione_zyczenia",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.niespelnione_zyczenia,
                        ),
                    ),
                    (
                        "rozbieznosci",
                        xmeta_room::ToSqlValue::to_sql_value(&params.rozbieznosci),
                    ),
                    ("numer_kw", xmeta_room::ToSqlValue::to_sql_value(&params.numer_kw)),
                    (
                        "cena_ogloszenia",
                        xmeta_room::ToSqlValue::to_sql_value(&params.cena_ogloszenia),
                    ),
                    (
                        "ceny_rcn_dokument_id",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.ceny_rcn_dokument_id,
                        ),
                    ),
                    (
                        "utworzono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.utworzono),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_przegladanie_ksiegi_propozycji`.
#[derive(Debug, Clone, PartialEq)]
pub struct OdnotujPrzegladanieKsiegiPropozycjiParams {
    pub numer_kw: String,
    pub kw_przegladano: String,
    pub kw_ogloszajacy_wpisany: WpisyKwOgloszajacyWpisany,
    pub kw_wpisy_iii_iv: Option<String>,
    pub id: String,
}

/// Wynik przeglądania księgi oglądanej nieruchomości, bez oceny skutków; wpisuje się go raz.
pub fn odnotuj_przegladanie_ksiegi_propozycji(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujPrzegladanieKsiegiPropozycjiParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE poszukiwanie_propozycje_wpisy
SET numer_kw = :numer_kw,
  kw_przegladano = :kw_przegladano,
  kw_ogloszajacy_wpisany = :kw_ogloszajacy_wpisany,
  kw_wpisy_iii_iv = :kw_wpisy_iii_iv
WHERE id = :id"#,
                parameters: Vec::from([
                    ("numer_kw", xmeta_room::ToSqlValue::to_sql_value(&params.numer_kw)),
                    (
                        "kw_przegladano",
                        xmeta_room::ToSqlValue::to_sql_value(&params.kw_przegladano),
                    ),
                    (
                        "kw_ogloszajacy_wpisany",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.kw_ogloszajacy_wpisany,
                        ),
                    ),
                    (
                        "kw_wpisy_iii_iv",
                        xmeta_room::ToSqlValue::to_sql_value(&params.kw_wpisy_iii_iv),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_ceny_transakcji_propozycji`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujCenyTransakcjiPropozycjiParams {
    pub ceny_rcn_dokument_id: String,
    pub id: String,
}

/// Dokument z cenami podobnych transakcji z rejestru cen nieruchomości, który Opiekun pokazał
/// klientowi.
pub fn odnotuj_ceny_transakcji_propozycji(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujCenyTransakcjiPropozycjiParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE poszukiwanie_propozycje_wpisy
SET ceny_rcn_dokument_id = :ceny_rcn_dokument_id
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "ceny_rcn_dokument_id",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.ceny_rcn_dokument_id,
                        ),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_rozbieznosci_propozycji`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujRozbieznosciPropozycjiParams {
    pub rozbieznosci: String,
    pub id: String,
}

/// Rozbieżności zauważone jako fakt, bez oceny; wpisuje się je raz.
pub fn odnotuj_rozbieznosci_propozycji(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujRozbieznosciPropozycjiParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE poszukiwanie_propozycje_wpisy
SET rozbieznosci = :rozbieznosci
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "rozbieznosci",
                        xmeta_room::ToSqlValue::to_sql_value(&params.rozbieznosci),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_przedstawienie_propozycji`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujPrzedstawieniePropozycjiParams {
    pub przedstawiono: String,
    pub id: String,
}

/// Chwila, w której klient dostał propozycję; nie wcześniej niż praca według umowy.
pub fn odnotuj_przedstawienie_propozycji(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujPrzedstawieniePropozycjiParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE poszukiwanie_propozycje_wpisy
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

/// The parameters of `odnotuj_reakcje_na_propozycje`.
#[derive(Debug, Clone, PartialEq)]
pub struct OdnotujReakcjeNaPropozycjeParams {
    pub reakcja: WpisyReakcja,
    pub reakcja_powod: Option<String>,
    pub id: String,
}

/// Reakcja klienta z powodem; po „czeka” klient może odpowiedzieć jeszcze raz.
pub fn odnotuj_reakcje_na_propozycje(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujReakcjeNaPropozycjeParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE poszukiwanie_propozycje_wpisy
SET reakcja = :reakcja,
  reakcja_powod = :reakcja_powod
WHERE id = :id"#,
                parameters: Vec::from([
                    ("reakcja", xmeta_room::ToSqlValue::to_sql_value(&params.reakcja)),
                    (
                        "reakcja_powod",
                        xmeta_room::ToSqlValue::to_sql_value(&params.reakcja_powod),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}
