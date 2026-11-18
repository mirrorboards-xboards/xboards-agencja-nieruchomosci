-- Agenci, których powołał kierownik działu, i każde wpuszczenie agenta do pokoju, które kierownik
-- zrobił albo, w Kontroli, o które poprosił.
--
-- Agent sprawy pracuje w jednej teczce. Koordynator operacji nie ma teczki, bo obsługuje lokale
-- obszaru (obszar wskazuje go w pamięci Kierownika najmu). Kontroler nie ma teczki, bo pracuje dla
-- klienta, a do teczek klienta wpuszczają go kierownicy działów.
--
-- Wymiana jest ostateczna i wskazuje następcę. Następca przejmuje tę samą pracę: to samo
-- stanowisko, tę samą teczkę, a u Kontrolera tego samego klienta.
--
-- W okresie wdrożenia człowiek zatwierdza każde powołanie i każde wpuszczenie. Jego podpis
-- przychodzi wiadomością, a kierownik przenosi go do wiersza; sprawdza go aplikacja. Kiedy okres
-- wdrożenia trwa, ustala właściciel, więc baza wymaga tylko kompletu kolumn zatwierdzenia.
CREATE TABLE kierownik_dzialu_agenci_wpisy (
  id TEXT PRIMARY KEY,
  stanowisko TEXT NOT NULL CHECK (stanowisko IN ('opiekun_oferty', 'opiekun_poszukujacego', 'analityk_kredytowy', 'analityk_stanu_prawnego', 'koordynator_transakcji', 'opiekun_najmu', 'zarzadca_obiektu', 'konsjerz', 'koordynator_operacji', 'kontroler')),
  tozsamosc TEXT NOT NULL,
  -- Człowiek, który jest właścicielem agenta, np. pośrednik, który założył teczkę oferty.
  wlasciciel TEXT NOT NULL,
  -- Pokój teczki, tekstem.
  teczka_odnosnik TEXT,
  -- Klient Kontrolera: wpis w kartotece klientów, tekstem.
  klient_odnosnik TEXT,
  powolano TEXT NOT NULL,
  stan TEXT NOT NULL CHECK (stan IN ('pracuje', 'spi', 'wymieniony')),
  wymieniono TEXT,
  powod_wymiany TEXT,
  nastepca_id TEXT REFERENCES kierownik_dzialu_agenci_wpisy (id),
  zatwierdzil TEXT,
  podpis TEXT,
  zatwierdzono TEXT,

  CONSTRAINT teczka_poza_koordynatorem_operacji_i_kontrolerem CHECK ((stanowisko IN ('koordynator_operacji', 'kontroler')) = (teczka_odnosnik IS NULL)),
  CONSTRAINT klient_tylko_u_kontrolera CHECK ((stanowisko = 'kontroler') = (klient_odnosnik IS NOT NULL)),
  CONSTRAINT wymiana_z_powodem_data_i_nastepca CHECK (
    (stan = 'wymieniony') = (wymieniono IS NOT NULL)
    AND (stan = 'wymieniony') = (powod_wymiany IS NOT NULL)
    AND (stan = 'wymieniony') = (nastepca_id IS NOT NULL)
  ),
  CONSTRAINT zatwierdzenie_w_komplecie CHECK (
    (zatwierdzil IS NULL) = (podpis IS NULL)
    AND (zatwierdzil IS NULL) = (zatwierdzono IS NULL)
  )
) STRICT;

-- Każde wpuszczenie agenta do teczki albo pokoju wymiany, także agenta innego działu (np. Kontrolera
-- na wskazanie Kierownika kontroli albo Analityka kredytowego na prośbę Kierownika transakcji).
-- Agent tego kierownika ma wskazany swój wiersz. Kto poprosił, to kierownik, który wskazał agenta,
-- a jego prośba to wiadomość; kto wpuścił, to kierownik, który dodał agenta do pokoju.
CREATE TABLE kierownik_dzialu_agenci_wpuszczenia (
  -- Klucz nadany przez bazę jest dodatni; zob. kierownik_dzialu_agenci_wpuszczenia_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  tozsamosc TEXT NOT NULL,
  stanowisko TEXT NOT NULL CHECK (stanowisko IN ('opiekun_oferty', 'opiekun_poszukujacego', 'analityk_kredytowy', 'analityk_stanu_prawnego', 'koordynator_transakcji', 'opiekun_najmu', 'zarzadca_obiektu', 'konsjerz', 'koordynator_operacji', 'kontroler')),
  agent_id TEXT REFERENCES kierownik_dzialu_agenci_wpisy (id),
  -- Pokój teczki albo pokój wymiany, tekstem.
  pokoj_odnosnik TEXT NOT NULL CHECK (pokoj_odnosnik <> ''),
  -- Klient Kontrolera: wpis w kartotece klientów, tekstem.
  klient_odnosnik TEXT,
  prosil TEXT,
  prosba_odnosnik TEXT,
  wpuscil TEXT NOT NULL,
  wpuszczono TEXT NOT NULL,
  usunieto TEXT,
  zatwierdzil TEXT,
  podpis TEXT,
  zatwierdzono TEXT,

  CONSTRAINT klient_tylko_u_kontrolera CHECK ((stanowisko = 'kontroler') = (klient_odnosnik IS NOT NULL)),
  CONSTRAINT prosba_razem_z_proszacym CHECK ((prosil IS NULL) = (prosba_odnosnik IS NULL)),
  CONSTRAINT usuniecie_nie_przed_wpuszczeniem CHECK (usunieto IS NULL OR usunieto >= wpuszczono),
  CONSTRAINT zatwierdzenie_w_komplecie CHECK (
    (zatwierdzil IS NULL) = (podpis IS NULL)
    AND (zatwierdzil IS NULL) = (zatwierdzono IS NULL)
  )
) STRICT;

-- Tożsamość jest w pokoju raz, więc otwarte wpuszczenie jednej tożsamości do jednego pokoju jest
-- jedno; ponowne wpuszczenie przychodzi po usunięciu.
CREATE UNIQUE INDEX kierownik_dzialu_agenci_jedno_otwarte_wpuszczenie
ON kierownik_dzialu_agenci_wpuszczenia (tozsamosc, pokoj_odnosnik)
WHERE usunieto IS NULL;

-- Drugie otwarte wpuszczenie z innym kluczem zderzyłoby się z indeksem, a INSERT OR REPLACE usunąłby
-- wtedy pierwsze z ominięciem blokady usuwania; ten trigger odmawia wcześniej.
CREATE TRIGGER kierownik_dzialu_agenci_jedno_otwarte_wpuszczenie_do_pokoju
BEFORE INSERT ON kierownik_dzialu_agenci_wpuszczenia
WHEN NEW.usunieto IS NULL
  AND EXISTS (
    SELECT 1
    FROM kierownik_dzialu_agenci_wpuszczenia
    WHERE tozsamosc = NEW.tozsamosc
      AND pokoj_odnosnik = NEW.pokoj_odnosnik
      AND usunieto IS NULL
      AND id IS NOT NEW.id
  )
BEGIN
  SELECT RAISE(ABORT, 'ta tożsamość ma już otwarte wpuszczenie do tego pokoju; nowe przychodzi po usunięciu');
END;

-- Agenci -----------------------------------------------------------------------------------------

-- Reguła następcy sprawdza wymianę, więc agent nie może powstać od razu wymieniony.
CREATE TRIGGER kierownik_dzialu_agenci_powstaje_pracujacy
BEFORE INSERT ON kierownik_dzialu_agenci_wpisy
WHEN NEW.stan <> 'pracuje'
BEGIN
  SELECT RAISE(ABORT, 'agent powstaje jako pracujący');
END;

CREATE TRIGGER kierownik_dzialu_agenci_wymiana_jest_ostateczna
BEFORE UPDATE ON kierownik_dzialu_agenci_wpisy
WHEN OLD.stan = 'wymieniony'
BEGIN
  SELECT RAISE(ABORT, 'wymienionego agenta się nie zmienia');
END;

-- Na stanowisku, teczce i kliencie stoi reguła następcy, a zatwierdzenie obejmuje powołanie.
CREATE TRIGGER kierownik_dzialu_agenci_powolanie_niezmienne
BEFORE UPDATE ON kierownik_dzialu_agenci_wpisy
WHEN NEW.id IS NOT OLD.id
  OR NEW.stanowisko IS NOT OLD.stanowisko
  OR NEW.tozsamosc IS NOT OLD.tozsamosc
  OR NEW.teczka_odnosnik IS NOT OLD.teczka_odnosnik
  OR NEW.klient_odnosnik IS NOT OLD.klient_odnosnik
  OR NEW.powolano IS NOT OLD.powolano
  OR (OLD.zatwierdzil IS NOT NULL AND (
    NEW.zatwierdzil IS NOT OLD.zatwierdzil
    OR NEW.podpis IS NOT OLD.podpis
    OR NEW.zatwierdzono IS NOT OLD.zatwierdzono
  ))
BEGIN
  SELECT RAISE(ABORT, 'stanowiska, tożsamości, teczki, klienta, daty powołania ani zatwierdzenia agenta się nie zmienia');
END;

CREATE TRIGGER kierownik_dzialu_agenci_nastepca_przejmuje_te_sama_prace
BEFORE UPDATE OF nastepca_id ON kierownik_dzialu_agenci_wpisy
WHEN NEW.nastepca_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM kierownik_dzialu_agenci_wpisy AS nastepca
    WHERE nastepca.id = NEW.nastepca_id
      AND nastepca.id <> NEW.id
      AND nastepca.stanowisko = NEW.stanowisko
      AND nastepca.teczka_odnosnik IS NEW.teczka_odnosnik
      AND nastepca.klient_odnosnik IS NEW.klient_odnosnik
  )
BEGIN
  SELECT RAISE(ABORT, 'następca ma to samo stanowisko i tę samą teczkę, a Kontroler tego samego klienta');
END;

-- Wpuszczenia ------------------------------------------------------------------------------------

CREATE TRIGGER kierownik_dzialu_agenci_wpuszczenie_powstaje_otwarte
BEFORE INSERT ON kierownik_dzialu_agenci_wpuszczenia
WHEN NEW.usunieto IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'wpuszczenie powstaje otwarte; usunięcie odnotowuje się później');
END;

-- Wpuszczenie własnego agenta powtarza jego tożsamość, stanowisko i klienta, bo te kolumny wpuszczeń
-- agentów innych działów są jedynym zapisem, kogo kierownik wpuścił.
CREATE TRIGGER kierownik_dzialu_agenci_wpuszczenie_zgodne_z_agentem
BEFORE INSERT ON kierownik_dzialu_agenci_wpuszczenia
WHEN NEW.agent_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM kierownik_dzialu_agenci_wpisy AS agent
    WHERE agent.id = NEW.agent_id
      AND agent.tozsamosc = NEW.tozsamosc
      AND agent.stanowisko = NEW.stanowisko
      AND agent.klient_odnosnik IS NEW.klient_odnosnik
  )
BEGIN
  SELECT RAISE(ABORT, 'wpuszczenie własnego agenta ma jego tożsamość, stanowisko i klienta');
END;

-- Wpuszczenie wolno raz zamknąć usunięciem i raz zatwierdzić; reszta jest śladem.
CREATE TRIGGER kierownik_dzialu_agenci_wpuszczenia_bez_zmian
BEFORE UPDATE ON kierownik_dzialu_agenci_wpuszczenia
WHEN NEW.id IS NOT OLD.id
  OR NEW.tozsamosc IS NOT OLD.tozsamosc
  OR NEW.stanowisko IS NOT OLD.stanowisko
  OR NEW.agent_id IS NOT OLD.agent_id
  OR NEW.pokoj_odnosnik IS NOT OLD.pokoj_odnosnik
  OR NEW.klient_odnosnik IS NOT OLD.klient_odnosnik
  OR NEW.prosil IS NOT OLD.prosil
  OR NEW.prosba_odnosnik IS NOT OLD.prosba_odnosnik
  OR NEW.wpuscil IS NOT OLD.wpuscil
  OR NEW.wpuszczono IS NOT OLD.wpuszczono
  OR (OLD.usunieto IS NOT NULL AND NEW.usunieto IS NOT OLD.usunieto)
  OR (OLD.zatwierdzil IS NOT NULL AND (
    NEW.zatwierdzil IS NOT OLD.zatwierdzil
    OR NEW.podpis IS NOT OLD.podpis
    OR NEW.zatwierdzono IS NOT OLD.zatwierdzono
  ))
BEGIN
  SELECT RAISE(ABORT, 'wpuszczenia się nie zmienia; wolno raz odnotować usunięcie i raz zatwierdzenie');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers. Bez podanego klucza
-- NEW.id tabeli z kluczem liczbowym jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- są dodatnie.
CREATE TRIGGER kierownik_dzialu_agenci_wpisy_bez_zastepowania
BEFORE INSERT ON kierownik_dzialu_agenci_wpisy
WHEN EXISTS (
  SELECT 1
  FROM kierownik_dzialu_agenci_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

CREATE TRIGGER kierownik_dzialu_agenci_wpuszczenia_bez_zastepowania
BEFORE INSERT ON kierownik_dzialu_agenci_wpuszczenia
WHEN EXISTS (
  SELECT 1
  FROM kierownik_dzialu_agenci_wpuszczenia
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- R9: wymieniony agent i zamknięte wpuszczenie zostają jako ślad.
CREATE TRIGGER kierownik_dzialu_agenci_wpisy_bez_usuwania
BEFORE DELETE ON kierownik_dzialu_agenci_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;

CREATE TRIGGER kierownik_dzialu_agenci_wpuszczenia_bez_usuwania
BEFORE DELETE ON kierownik_dzialu_agenci_wpuszczenia
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;
