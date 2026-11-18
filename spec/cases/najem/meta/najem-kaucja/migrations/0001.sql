-- Kaucja najmu: pobranie, wykorzystanie na zaległość przy najmie instytucjonalnym i rozliczenie
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
