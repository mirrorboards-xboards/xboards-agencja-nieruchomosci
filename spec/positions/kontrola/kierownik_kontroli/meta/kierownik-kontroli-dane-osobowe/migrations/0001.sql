-- Dane osobowe: prośby osób o ich dane (art. 15–18, 20 i 21 RODO) i zgłoszone naruszenia ochrony
-- danych, z datą, terminem, stanem i odwołaniem do odpowiedzi właściciela. Kierownik przygotowuje
-- odpowiedź albo opis naruszenia, a załatwia sprawę właściciel.
--
-- Treść prośby leży w wiadomości, którą wpłynęła; wiersz ją tylko wskazuje. Prośby o usunięcie
-- danych (art. 17 RODO) pamięć nie wykona na własnych wierszach, dopóki obowiązuje rozstrzygnięcie
-- biura R9.
CREATE TABLE kierownik_kontroli_dane_osobowe_wpisy (
  -- Klucz nadany przez bazę jest dodatni; zob. kierownik_kontroli_dane_osobowe_wpisy_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('dostep_art15', 'sprostowanie_art16', 'usuniecie_art17', 'ograniczenie_art18', 'przenoszenie_art20', 'sprzeciw_art21', 'naruszenie')),
  wplynelo TEXT NOT NULL,
  -- Wiadomość z prośbą albo zgłoszenie naruszenia, tekstem.
  wplyw_odnosnik TEXT NOT NULL CHECK (wplyw_odnosnik <> ''),
  -- Termin, który policzył program, np. miesiąc na odpowiedź (art. 12 ust. 3 RODO) albo 72 godziny
  -- na zgłoszenie naruszenia (art. 33 ust. 1 RODO).
  termin TEXT NOT NULL,
  stan TEXT NOT NULL CHECK (stan IN ('otwarta', 'u_wlasciciela', 'zalatwiona')),
  -- Odpowiedź albo zgłoszenie, które wysłał właściciel, tekstem.
  odpowiedz_odnosnik TEXT,
  zalatwiono TEXT,

  CONSTRAINT termin_nie_przed_wplywem CHECK (termin >= wplynelo),
  CONSTRAINT zalatwiona_z_odpowiedzia_wlasciciela CHECK (
    (stan = 'zalatwiona') = (odpowiedz_odnosnik IS NOT NULL)
    AND (stan = 'zalatwiona') = (zalatwiono IS NOT NULL)
  ),
  CONSTRAINT zalatwienie_nie_przed_wplywem CHECK (zalatwiono IS NULL OR zalatwiono >= wplynelo)
) STRICT;

CREATE TRIGGER kierownik_kontroli_dane_osobowe_powstaje_otwarta
BEFORE INSERT ON kierownik_kontroli_dane_osobowe_wpisy
WHEN NEW.stan <> 'otwarta'
BEGIN
  SELECT RAISE(ABORT, 'sprawa danych osobowych powstaje otwarta');
END;

CREATE TRIGGER kierownik_kontroli_dane_osobowe_bez_zmian
BEFORE UPDATE ON kierownik_kontroli_dane_osobowe_wpisy
WHEN OLD.stan = 'zalatwiona'
  OR NEW.id IS NOT OLD.id
  OR NEW.rodzaj IS NOT OLD.rodzaj
  OR NEW.wplynelo IS NOT OLD.wplynelo
  OR NEW.wplyw_odnosnik IS NOT OLD.wplyw_odnosnik
BEGIN
  SELECT RAISE(ABORT, 'rodzaju ani wpływu sprawy danych osobowych się nie zmienia, a załatwionej wcale');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers. Bez podanego klucza
-- NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze są dodatnie.
CREATE TRIGGER kierownik_kontroli_dane_osobowe_wpisy_bez_zastepowania
BEFORE INSERT ON kierownik_kontroli_dane_osobowe_wpisy
WHEN EXISTS (
  SELECT 1
  FROM kierownik_kontroli_dane_osobowe_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- R9: załatwione sprawy zostają jako ślad dla organu nadzorczego.
CREATE TRIGGER kierownik_kontroli_dane_osobowe_wpisy_bez_usuwania
BEFORE DELETE ON kierownik_kontroli_dane_osobowe_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;
