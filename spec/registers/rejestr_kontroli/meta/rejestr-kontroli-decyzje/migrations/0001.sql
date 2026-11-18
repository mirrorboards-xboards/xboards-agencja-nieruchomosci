-- Prośby Kierownika kontroli o rozstrzygnięcie człowieka i rozstrzygnięcia podpisane jego kluczem:
-- zatwierdzenie oceny ryzyka klienta, zdjęcie wątpliwości, potwierdzenie podejrzenia, przekazanie
-- zawiadomienia, akceptacje właściciela biura i decyzja o nienawiązaniu stosunków. Inne rodzaje
-- rejestru wskazują decyzję kluczem obcym, więc decyzje są pierwszym rodzajem rejestru.
--
-- Przebieg jest jak w teczce (sprawa-decyzje): prośba powstaje jako czekająca, rozstrzygnięcie ma
-- komplet podpisu, a rozstrzygniętej ani zbędnej decyzji się nie zmienia. Baza wymaga kompletu
-- kolumn podpisu; sam podpis kluczem xkeys sprawdza aplikacja. Inna decyzja w tej samej sprawie
-- to nowa prośba.
--
-- Rejestr czytają tylko Kierownik kontroli, właściciel biura i osoba wyznaczona (art. 54 ust. 1
-- AML). Nic z niego nie wraca do teczek; decyzja w teczce wskazuje co najwyżej zdjęcie znacznika.
CREATE TABLE rejestr_kontroli_decyzje_wpisy (
  id TEXT PRIMARY KEY,
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('zatwierdzenie_oceny', 'zdjecie_watpliwosci', 'potwierdzenie_podejrzenia', 'przekazanie_zawiadomienia', 'akceptacja_pep', 'akceptacja_procedury', 'akceptacja_oceny_biura', 'nienawiazanie_stosunkow')),
  -- Kierownik kontroli, który prosi o rozstrzygnięcie.
  prosi TEXT NOT NULL,
  -- Człowiek, który rozstrzyga, i jego funkcja: właściciel biura (art. 6 AML) albo osoba
  -- wyznaczona do obowiązków z ustawy (art. 8 AML).
  decyduje TEXT NOT NULL,
  decyduje_funkcja TEXT NOT NULL CHECK (decyduje_funkcja IN ('wlasciciel_biura', 'osoba_wyznaczona')),
  zlozono TEXT NOT NULL,
  -- Czego prośba dotyczy, krótko.
  tresc TEXT,
  -- Wiersz rejestru, którego dotyczy decyzja; jego identyfikator nadaje aplikacja, zanim powstanie
  -- prośba, więc decyzja może poprzedzać wiersz, który zatwierdza.
  dotyczy_tabela TEXT,
  dotyczy_id TEXT,
  -- Skrót wiersza z plikami, który liczy aplikacja; obejmuje go podpis.
  dotyczy_skrot TEXT,
  stan TEXT NOT NULL CHECK (stan IN ('czeka', 'rozstrzygnieta', 'zbedna')),
  rozstrzygniecie TEXT CHECK (rozstrzygniecie IN ('zgoda', 'odmowa')),
  uzasadnienie TEXT,
  podpisal TEXT,
  podpis TEXT,
  podpisano TEXT,
  powod_zbednosci TEXT,
  -- Decyzja zdjecie_watpliwosci w teczce (sprawa_decyzje_wpisy), bez treści, która zdejmuje tam
  -- znacznik; podstawy są tylko tutaj.
  teczka_decyzja_odnosnik TEXT,

  CONSTRAINT podpis_w_komplecie CHECK (
    (podpisal IS NULL) = (rozstrzygniecie IS NULL)
    AND (podpisal IS NULL) = (podpis IS NULL)
    AND (podpisal IS NULL) = (podpisano IS NULL)
  ),
  CONSTRAINT podpis_tylko_przy_rozstrzygnieciu CHECK ((stan = 'rozstrzygnieta') = (podpisal IS NOT NULL)),
  -- Rozstrzygnięcia wpisuje tylko człowiek swoim kluczem; Kierownik kontroli nie rozstrzyga za niego.
  CONSTRAINT podpisuje_ten_kto_decyduje CHECK (podpisal IS NULL OR podpisal = decyduje),
  CONSTRAINT rozstrzygniecie_nie_przed_zlozeniem CHECK (podpisano IS NULL OR podpisano >= zlozono),
  CONSTRAINT dotyczy_tabela_razem_z_id CHECK ((dotyczy_tabela IS NULL) = (dotyczy_id IS NULL)),
  CONSTRAINT zbedna_ma_powod CHECK ((stan = 'zbedna') = (powod_zbednosci IS NOT NULL)),
  -- Art. 46 ust. 2 pkt 1 i art. 50 ust. 3 AML oraz reguła biura: stosunki z osobą zajmującą
  -- eksponowane stanowisko polityczne, procedurę i ocenę ryzyka biura akceptuje właściciel biura.
  -- Pozostałe rozstrzygnięcia, poza decyzją o nienawiązaniu stosunków, należą do osoby wyznaczonej.
  CONSTRAINT rozstrzyga_wlasciwa_osoba CHECK (
    CASE
      WHEN rodzaj IN ('akceptacja_pep', 'akceptacja_procedury', 'akceptacja_oceny_biura') THEN decyduje_funkcja = 'wlasciciel_biura'
      WHEN rodzaj = 'nienawiazanie_stosunkow' THEN 1
      ELSE decyduje_funkcja = 'osoba_wyznaczona'
    END
  ),
  CONSTRAINT zdjecie_watpliwosci_z_uzasadnieniem CHECK (
    rodzaj <> 'zdjecie_watpliwosci' OR rozstrzygniecie IS NULL OR uzasadnienie IS NOT NULL
  ),
  CONSTRAINT decyzja_w_teczce_tylko_przy_zdjeciu_watpliwosci CHECK (
    teczka_decyzja_odnosnik IS NULL OR rodzaj = 'zdjecie_watpliwosci'
  )
) STRICT;

-- Zatwierdzone decyzje: rozstrzygnięte zgodą. Reguły pozostałych rodzajów rejestru czytają ten
-- widok, gdy wiersz wymaga rozstrzygnięcia człowieka.
CREATE VIEW rejestr_kontroli_decyzje_zatwierdzone AS
SELECT *
FROM rejestr_kontroli_decyzje_wpisy
WHERE stan = 'rozstrzygnieta'
  AND rozstrzygniecie = 'zgoda';

-- Reguły rozstrzygnięcia sprawdzają zmianę stanu, więc prośba nie może powstać od razu
-- rozstrzygnięta.
CREATE TRIGGER rejestr_kontroli_decyzje_prosba_powstaje_jako_czekajaca
BEFORE INSERT ON rejestr_kontroli_decyzje_wpisy
WHEN NEW.stan <> 'czeka'
BEGIN
  SELECT RAISE(ABORT, 'prośba o decyzję powstaje jako czekająca');
END;

-- Prośbę rozstrzyga się albo uznaje za zbędną; jej treści się nie zmienia, bo podpis obejmuje to,
-- o co prosiła.
CREATE TRIGGER rejestr_kontroli_decyzje_prosba_bez_zmian
BEFORE UPDATE ON rejestr_kontroli_decyzje_wpisy
WHEN NEW.id IS NOT OLD.id
  OR NEW.rodzaj IS NOT OLD.rodzaj
  OR NEW.prosi IS NOT OLD.prosi
  OR NEW.decyduje IS NOT OLD.decyduje
  OR NEW.decyduje_funkcja IS NOT OLD.decyduje_funkcja
  OR NEW.zlozono IS NOT OLD.zlozono
  OR NEW.tresc IS NOT OLD.tresc
  OR NEW.dotyczy_tabela IS NOT OLD.dotyczy_tabela
  OR NEW.dotyczy_id IS NOT OLD.dotyczy_id
  OR NEW.dotyczy_skrot IS NOT OLD.dotyczy_skrot
BEGIN
  SELECT RAISE(ABORT, 'prośby o decyzję się nie zmienia; inna sprawa to nowa prośba');
END;

CREATE TRIGGER rejestr_kontroli_decyzje_rozstrzygniecie_ostateczne
BEFORE UPDATE ON rejestr_kontroli_decyzje_wpisy
WHEN OLD.stan <> 'czeka'
BEGIN
  SELECT RAISE(ABORT, 'rozstrzygniętej ani zbędnej decyzji się nie zmienia; inna decyzja to nowa prośba');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers. Bez podanego klucza
-- NEW.id tabel z kluczem liczbowym jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- są dodatnie.
CREATE TRIGGER rejestr_kontroli_decyzje_wpisy_bez_zastepowania
BEFORE INSERT ON rejestr_kontroli_decyzje_wpisy
WHEN EXISTS (
  SELECT 1
  FROM rejestr_kontroli_decyzje_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza rejestru nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

-- Art. 49 AML i R9: dokumentację przechowuje się co najmniej 5 lat, a na razie bez terminu.
CREATE TRIGGER rejestr_kontroli_decyzje_wpisy_bez_usuwania
BEFORE DELETE ON rejestr_kontroli_decyzje_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy rejestru się nie usuwa (R9); błąd poprawia nowy wpis');
END;
