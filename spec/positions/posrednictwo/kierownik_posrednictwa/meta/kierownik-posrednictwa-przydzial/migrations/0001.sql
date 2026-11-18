-- Przydział pośredników: reguły każdego pośrednika (obszar, rodzaje nieruchomości, limit spraw
-- i czy przyjmuje nowe) i karty od Kierownika biura z pośrednikiem, którego kierownik zaproponował.
--
-- Reguły pośrednika zmienia tylko jego wiadomość podpisana jego kluczem (podszywanie się); podpis
-- sprawdza aplikacja, a baza wymaga jego obecności. Zmiana to nowy wiersz, a poprzedni dostaje datę
-- zastąpienia, więc historia zostaje (R9). Pośrednik ma naraz jedną regułę.
CREATE TABLE kierownik_posrednictwa_przydzial_posrednicy (
  -- Klucz nadany przez bazę jest dodatni; zob. kierownik_posrednictwa_przydzial_posrednicy_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  -- Tożsamość pośrednika.
  posrednik TEXT NOT NULL CHECK (posrednik <> ''),
  -- Np. „Wrocław, Krzyki i Fabryczna”.
  obszar TEXT NOT NULL CHECK (obszar <> ''),
  -- Np. „mieszkania, domy”.
  rodzaje_nieruchomosci TEXT NOT NULL CHECK (rodzaje_nieruchomosci <> ''),
  -- Ile spraw (ofert i poszukiwań) pośrednik prowadzi naraz najwyżej.
  limit_spraw INTEGER NOT NULL CHECK (limit_spraw > 0),
  przyjmuje_nowe INTEGER NOT NULL CHECK (przyjmuje_nowe IN (0, 1)),
  od TEXT NOT NULL,
  -- Podpisana wiadomość pośrednika, która ustaliła reguły, tekstem.
  wiadomosc_odnosnik TEXT NOT NULL CHECK (wiadomosc_odnosnik <> ''),
  podpis TEXT NOT NULL CHECK (podpis <> ''),
  zastapione_od TEXT,

  CONSTRAINT zastapienie_nie_przed_poczatkiem CHECK (zastapione_od IS NULL OR zastapione_od >= od)
) STRICT;

-- Karta nowej sprawy od Kierownika biura. Kontakt leży na karcie, dopóki pośrednik nie założy
-- teczki albo karta nie zostanie odrzucona; potem kierownik go usuwa, bo trafił do teczki.
-- Pośrednik spoza reguł wymaga zgody człowieka; jego podpis kierownik przenosi do wiersza.
CREATE TABLE kierownik_posrednictwa_przydzial_karty (
  -- Klucz nadany przez bazę jest dodatni; zob. kierownik_posrednictwa_przydzial_karty_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  -- Karta w pamięci Kierownika biura, tekstem.
  karta_odnosnik TEXT NOT NULL UNIQUE CHECK (karta_odnosnik <> ''),
  otrzymano TEXT NOT NULL,
  kontakt TEXT,
  stan TEXT NOT NULL CHECK (stan IN ('czeka', 'zaproponowana', 'przyjeta', 'odrzucona')),
  -- Tożsamość zaproponowanego pośrednika i reguła, według której go zaproponowano.
  posrednik TEXT,
  regula_id INTEGER REFERENCES kierownik_posrednictwa_przydzial_posrednicy (id),
  zaproponowano TEXT,
  -- Teczka oferty albo poszukiwania, którą pośrednik założył, tekstem.
  teczka_odnosnik TEXT,
  powod TEXT,
  rozstrzygnieto TEXT,
  zatwierdzil TEXT,
  podpis TEXT,
  zatwierdzono TEXT,

  CONSTRAINT propozycja_z_posrednikiem CHECK (
    (stan IN ('zaproponowana', 'przyjeta')) = (posrednik IS NOT NULL)
    AND (posrednik IS NULL) = (zaproponowano IS NULL)
  ),
  CONSTRAINT regula_przy_posredniku CHECK (regula_id IS NULL OR posrednik IS NOT NULL),
  CONSTRAINT posrednik_poza_regulami_za_zgoda_czlowieka CHECK (
    posrednik IS NULL
    OR regula_id IS NOT NULL
    OR zatwierdzil IS NOT NULL
  ),
  CONSTRAINT przyjeta_z_teczka CHECK ((stan = 'przyjeta') = (teczka_odnosnik IS NOT NULL)),
  CONSTRAINT odrzucona_z_powodem CHECK ((stan = 'odrzucona') = (powod IS NOT NULL)),
  CONSTRAINT rozstrzygnieta_z_data CHECK ((stan IN ('przyjeta', 'odrzucona')) = (rozstrzygnieto IS NOT NULL)),
  CONSTRAINT rozstrzygnieta_bez_kontaktu CHECK (stan NOT IN ('przyjeta', 'odrzucona') OR kontakt IS NULL),
  CONSTRAINT zatwierdzenie_w_komplecie CHECK (
    (zatwierdzil IS NULL) = (podpis IS NULL)
    AND (zatwierdzil IS NULL) = (zatwierdzono IS NULL)
  ),
  CONSTRAINT rozstrzygniecie_nie_przed_otrzymaniem CHECK (rozstrzygnieto IS NULL OR rozstrzygnieto >= otrzymano)
) STRICT;

-- Pośrednik ma naraz jedną regułę; nowa przychodzi po zastąpieniu starej.
CREATE UNIQUE INDEX kierownik_posrednictwa_przydzial_jedna_regula_posrednika
ON kierownik_posrednictwa_przydzial_posrednicy (posrednik)
WHERE zastapione_od IS NULL;

-- Pośrednicy ------------------------------------------------------------------------------------

CREATE TRIGGER kierownik_posrednictwa_przydzial_regula_powstaje_aktualna
BEFORE INSERT ON kierownik_posrednictwa_przydzial_posrednicy
WHEN NEW.zastapione_od IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'reguła pośrednika powstaje aktualna; zastąpienie odnotowuje się później');
END;

-- Druga aktualna reguła z innym kluczem zderzyłaby się z indeksem, a INSERT OR REPLACE usunąłby
-- wtedy pierwszą z ominięciem blokady usuwania; ten trigger odmawia wcześniej.
CREATE TRIGGER kierownik_posrednictwa_przydzial_jedna_regula_przed_zastapieniem
BEFORE INSERT ON kierownik_posrednictwa_przydzial_posrednicy
WHEN EXISTS (
  SELECT 1
  FROM kierownik_posrednictwa_przydzial_posrednicy
  WHERE posrednik = NEW.posrednik
    AND zastapione_od IS NULL
    AND id IS NOT NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'ten pośrednik ma już aktualną regułę; nowa przychodzi po zastąpieniu starej');
END;

-- Podpis obejmuje regułę, więc zmienia się tylko raz odnotowane zastąpienie.
CREATE TRIGGER kierownik_posrednictwa_przydzial_regula_bez_zmian
BEFORE UPDATE ON kierownik_posrednictwa_przydzial_posrednicy
WHEN OLD.zastapione_od IS NOT NULL
  OR NEW.id IS NOT OLD.id
  OR NEW.posrednik IS NOT OLD.posrednik
  OR NEW.obszar IS NOT OLD.obszar
  OR NEW.rodzaje_nieruchomosci IS NOT OLD.rodzaje_nieruchomosci
  OR NEW.limit_spraw IS NOT OLD.limit_spraw
  OR NEW.przyjmuje_nowe IS NOT OLD.przyjmuje_nowe
  OR NEW.od IS NOT OLD.od
  OR NEW.wiadomosc_odnosnik IS NOT OLD.wiadomosc_odnosnik
  OR NEW.podpis IS NOT OLD.podpis
BEGIN
  SELECT RAISE(ABORT, 'reguły pośrednika się nie zmienia; zmiana to nowa reguła z podpisem pośrednika');
END;

-- Karty ------------------------------------------------------------------------------------------

CREATE TRIGGER kierownik_posrednictwa_przydzial_karta_powstaje_czekajaca
BEFORE INSERT ON kierownik_posrednictwa_przydzial_karty
WHEN NEW.stan <> 'czeka'
BEGIN
  SELECT RAISE(ABORT, 'karta powstaje jako czekająca; propozycja pośrednika przychodzi później');
END;

-- Propozycja według reguły wskazuje pośrednika tej reguły, gdy reguła jest aktualna i pośrednik
-- przyjmuje nowe sprawy.
CREATE TRIGGER kierownik_posrednictwa_przydzial_propozycja_zgodna_z_regula
BEFORE UPDATE OF regula_id, posrednik ON kierownik_posrednictwa_przydzial_karty
WHEN NEW.regula_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM kierownik_posrednictwa_przydzial_posrednicy AS regula
    WHERE regula.id = NEW.regula_id
      AND regula.posrednik = NEW.posrednik
      AND regula.zastapione_od IS NULL
      AND regula.przyjmuje_nowe = 1
  )
BEGIN
  SELECT RAISE(ABORT, 'propozycja według reguły wskazuje pośrednika aktualnej reguły, który przyjmuje nowe sprawy');
END;

-- Karta od Kierownika biura się nie zmienia; przyjętej ani odrzuconej karty nie zmienia się wcale.
CREATE TRIGGER kierownik_posrednictwa_przydzial_karta_bez_zmian
BEFORE UPDATE ON kierownik_posrednictwa_przydzial_karty
WHEN OLD.stan IN ('przyjeta', 'odrzucona')
  OR NEW.id IS NOT OLD.id
  OR NEW.karta_odnosnik IS NOT OLD.karta_odnosnik
  OR NEW.otrzymano IS NOT OLD.otrzymano
BEGIN
  SELECT RAISE(ABORT, 'karty od Kierownika biura się nie zmienia, a przyjętej ani odrzuconej wcale');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers; także kartę o tym samym
-- odnośniku. Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo
-- klucze są dodatnie.
CREATE TRIGGER kierownik_posrednictwa_przydzial_posrednicy_bez_zastepowania
BEFORE INSERT ON kierownik_posrednictwa_przydzial_posrednicy
WHEN EXISTS (
  SELECT 1
  FROM kierownik_posrednictwa_przydzial_posrednicy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

CREATE TRIGGER kierownik_posrednictwa_przydzial_karty_bez_zastepowania
BEFORE INSERT ON kierownik_posrednictwa_przydzial_karty
WHEN EXISTS (
  SELECT 1
  FROM kierownik_posrednictwa_przydzial_karty
  WHERE id = NEW.id
    OR karta_odnosnik = NEW.karta_odnosnik
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- R9: zastąpione reguły i rozstrzygnięte karty zostają jako historia przydziałów.
CREATE TRIGGER kierownik_posrednictwa_przydzial_posrednicy_bez_usuwania
BEFORE DELETE ON kierownik_posrednictwa_przydzial_posrednicy
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;

CREATE TRIGGER kierownik_posrednictwa_przydzial_karty_bez_usuwania
BEFORE DELETE ON kierownik_posrednictwa_przydzial_karty
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;
