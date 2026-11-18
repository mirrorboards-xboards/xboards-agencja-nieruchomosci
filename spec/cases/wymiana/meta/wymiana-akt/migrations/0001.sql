-- Akt w pokoju wymiany: terminy aktu ze stanami obu stron, pierwokup i informacja po akcie.
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
