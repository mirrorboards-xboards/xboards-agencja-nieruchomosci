-- Karty dla Kierownika biura: nowe zgłoszenie, pismo, którego Recepcja nie umiała przypisać do
-- żadnej sprawy, albo skarga na biuro, i to, jak Kierownik biura kartę rozstrzygnął.
--
-- Karta ma stałe pola, które Recepcja wypełniła z wiadomości albo rozmowy. Wolny tekst
-- zgłaszającego zostaje w skrzynce, a karta ma do niego tylko odnośnik. Dane zgłaszającego leżą na
-- karcie tylko dopóki karta czeka. Po rozstrzygnięciu muszą zniknąć, bo trafiły do działu albo
-- zgłoszenie odrzucono, a Recepcja nie trzyma danych klientów. Poprzedni stan wiersza zostaje
-- w dzienniku pokoju.
CREATE TABLE recepcja_karty_wpisy (
  -- Klucz nadany przez bazę jest dodatni; zob. recepcja_karty_wpisy_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('nowe_zgloszenie', 'pismo_bez_sprawy', 'skarga')),
  -- Każda wiadomość ma wpis w dzienniku, także rozmowa telefoniczna.
  wplyw_id INTEGER NOT NULL REFERENCES recepcja_wplywy_dziennik (id),
  wyslano TEXT NOT NULL,
  -- Czego chce zgłaszający: sprzedać, kupić, wynająć albo oddać lokal w zarząd.
  rodzaj_sprawy TEXT CHECK (rodzaj_sprawy IN ('sprzedaz', 'kupno', 'wynajem', 'zarzad')),
  -- Np. „mieszkanie”, „dom”, „lokal użytkowy”.
  rodzaj_nieruchomosci TEXT,
  miejscowosc TEXT,
  -- Kanał, którym zgłaszający chce dostać odpowiedź.
  kanal TEXT CHECK (kanal IN ('email', 'formularz', 'portal', 'telefon', 'skan_poczty')),
  -- Kto się zgłasza i jak się z nim kontaktować; znika po rozstrzygnięciu.
  zglaszajacy TEXT,
  kontakt TEXT,
  -- Wolny tekst zgłaszającego w skrzynce, tekstem.
  wiadomosc_odnosnik TEXT NOT NULL CHECK (wiadomosc_odnosnik <> ''),
  stan TEXT NOT NULL CHECK (stan IN ('czeka', 'rozstrzygnieta')),
  rozstrzygnieto TEXT,
  rozstrzygniecie TEXT CHECK (rozstrzygniecie IN ('przydzielona', 'odrzucona')),
  -- Dział, któremu Kierownik biura przydzielił kartę.
  dzial TEXT CHECK (dzial IN ('posrednictwo', 'transakcje', 'najem')),

  CONSTRAINT czekajace_zgloszenie_z_danymi CHECK (
    stan <> 'czeka'
    OR rodzaj <> 'nowe_zgloszenie'
    OR (rodzaj_sprawy IS NOT NULL AND zglaszajacy IS NOT NULL AND kontakt IS NOT NULL)
  ),
  CONSTRAINT rozstrzygnieta_bez_danych_zglaszajacego CHECK (
    stan <> 'rozstrzygnieta'
    OR (zglaszajacy IS NULL AND kontakt IS NULL)
  ),
  CONSTRAINT rozstrzygnieta_z_data_i_sposobem CHECK (
    (stan = 'rozstrzygnieta') = (rozstrzygnieto IS NOT NULL)
    AND (stan = 'rozstrzygnieta') = (rozstrzygniecie IS NOT NULL)
  ),
  CONSTRAINT dzial_przy_przydziale CHECK ((rozstrzygniecie IS 'przydzielona') = (dzial IS NOT NULL)),
  CONSTRAINT rozstrzygniecie_nie_przed_wyslaniem CHECK (rozstrzygnieto IS NULL OR rozstrzygnieto >= wyslano)
) STRICT;

CREATE TRIGGER recepcja_karty_powstaje_czekajaca
BEFORE INSERT ON recepcja_karty_wpisy
WHEN NEW.stan <> 'czeka'
BEGIN
  SELECT RAISE(ABORT, 'karta powstaje jako czekająca; rozstrzyga ją Kierownik biura');
END;

-- Wpis dziennika mówi, komu oddano wiadomość, więc karta stoi na wpisie oddanym na kartę.
CREATE TRIGGER recepcja_karty_wplyw_oddany_na_karte
BEFORE INSERT ON recepcja_karty_wpisy
WHEN NOT EXISTS (
  SELECT 1
  FROM recepcja_wplywy_dziennik AS wplyw
  WHERE wplyw.id = NEW.wplyw_id
    AND wplyw.oddano = 'karta'
)
BEGIN
  SELECT RAISE(ABORT, 'karta stoi na wpisie dziennika oddanym na kartę');
END;

-- Czekającą kartę zmienia tylko rozstrzygnięcie; rozstrzygniętej nie zmienia się wcale.
CREATE TRIGGER recepcja_karty_bez_zmian
BEFORE UPDATE ON recepcja_karty_wpisy
WHEN OLD.stan = 'rozstrzygnieta'
  OR NEW.id IS NOT OLD.id
  OR NEW.rodzaj IS NOT OLD.rodzaj
  OR NEW.wplyw_id IS NOT OLD.wplyw_id
  OR NEW.wyslano IS NOT OLD.wyslano
  OR NEW.rodzaj_sprawy IS NOT OLD.rodzaj_sprawy
  OR NEW.rodzaj_nieruchomosci IS NOT OLD.rodzaj_nieruchomosci
  OR NEW.miejscowosc IS NOT OLD.miejscowosc
  OR NEW.kanal IS NOT OLD.kanal
  OR NEW.wiadomosc_odnosnik IS NOT OLD.wiadomosc_odnosnik
BEGIN
  SELECT RAISE(ABORT, 'wysłanej karty się nie zmienia; wolno ją raz rozstrzygnąć');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers. Bez podanego klucza
-- NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze są dodatnie.
CREATE TRIGGER recepcja_karty_wpisy_bez_zastepowania
BEFORE INSERT ON recepcja_karty_wpisy
WHEN EXISTS (
  SELECT 1
  FROM recepcja_karty_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- R9: rozstrzygnięta karta zostaje bez danych zgłaszającego.
CREATE TRIGGER recepcja_karty_wpisy_bez_usuwania
BEFORE DELETE ON recepcja_karty_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;
