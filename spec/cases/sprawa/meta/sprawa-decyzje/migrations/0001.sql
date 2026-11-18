-- Prośby agentów o decyzję człowieka i rozstrzygnięcia z podpisem. Inne wiersze teczki (i typy
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
