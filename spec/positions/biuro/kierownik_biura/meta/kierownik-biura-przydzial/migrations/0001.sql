-- Przydział nowych spraw: reguły, który rodzaj zgłoszenia trafia do którego działu, i przydziały
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
