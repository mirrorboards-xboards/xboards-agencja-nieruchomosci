-- Terminy sprawy: ustawowe z podstawą prawną, z umów i wewnętrzne biura. Liczy je program,
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
