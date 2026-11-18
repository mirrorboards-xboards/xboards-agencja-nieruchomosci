-- Kontakty: kto poza teczką zajmuje się sprawą, którym kanałem i o jakiej porze odpowiada, np.
-- administrator wspólnoty, kancelaria, bank, spółdzielnia albo wykonawca.
--
-- Uczestnika teczki albo wpis rejestru (np. wykonawcę z rejestru wykonawców) wskazuje odnośnik, bez
-- nazwy i adresu, bo te dane trzyma tamten pokój. Zmiana kontaktu to nowy wiersz, a stary dostaje
-- datę nieaktualności i zostaje jako historia.
CREATE TABLE stanowisko_kontakty_wpisy (
  -- Klucz nadany przez bazę jest dodatni; zob. stanowisko_kontakty_wpisy_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  -- Teczka albo teczka lokalu, której kontakt dotyczy, tekstem. Brak oznacza kontakt całego obszaru
  -- albo biura, np. dostawcę zapasów albo administrację budynku z wieloma lokalami.
  dotyczy_odnosnik TEXT CHECK (dotyczy_odnosnik IS NULL OR dotyczy_odnosnik <> ''),
  -- Np. „Wspólnota Mieszkaniowa Polna 7” albo „Bank Przykładowy”.
  instytucja TEXT,
  -- Np. „administrator”, „doradca kredytowy”, „sekretariat kancelarii”.
  rola TEXT NOT NULL CHECK (rola <> ''),
  -- Pokój i wiersz uczestnika teczki albo wpisu rejestru, tekstem.
  wpis_odnosnik TEXT,
  nazwa TEXT,
  -- Adres w kanale: adres poczty, numer telefonu albo adres pocztowy.
  adres TEXT,
  kanal TEXT NOT NULL CHECK (kanal IN ('email', 'telefon', 'sms', 'portal', 'kanal_sprzedazy', 'list', 'osobiscie', 'pokoj_wymiany')),
  -- Np. „dni robocze 8–12”.
  pory TEXT,
  zapisano TEXT NOT NULL,
  nieaktualny_od TEXT,

  CONSTRAINT kontakt_wskazany_albo_nazwany CHECK ((wpis_odnosnik IS NULL) <> (nazwa IS NULL)),
  CONSTRAINT nazwa_razem_z_adresem CHECK ((nazwa IS NULL) = (adres IS NULL)),
  CONSTRAINT nieaktualny_nie_przed_zapisem CHECK (nieaktualny_od IS NULL OR nieaktualny_od >= zapisano)
) STRICT;

-- Nieaktualność przychodzi przy zmianie kontaktu, więc kontakt powstaje aktualny.
CREATE TRIGGER stanowisko_kontakty_powstaje_aktualny
BEFORE INSERT ON stanowisko_kontakty_wpisy
WHEN NEW.nieaktualny_od IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'kontakt powstaje aktualny; zmiana kontaktu to nowy wiersz');
END;

CREATE TRIGGER stanowisko_kontakty_bez_zmian
BEFORE UPDATE ON stanowisko_kontakty_wpisy
WHEN OLD.nieaktualny_od IS NOT NULL
  OR NEW.id IS NOT OLD.id
  OR NEW.dotyczy_odnosnik IS NOT OLD.dotyczy_odnosnik
  OR NEW.instytucja IS NOT OLD.instytucja
  OR NEW.rola IS NOT OLD.rola
  OR NEW.wpis_odnosnik IS NOT OLD.wpis_odnosnik
  OR NEW.nazwa IS NOT OLD.nazwa
  OR NEW.adres IS NOT OLD.adres
  OR NEW.kanal IS NOT OLD.kanal
  OR NEW.pory IS NOT OLD.pory
  OR NEW.zapisano IS NOT OLD.zapisano
BEGIN
  SELECT RAISE(ABORT, 'kontaktu się nie zmienia; zmiana kontaktu to nowy wiersz, a stary staje się nieaktualny');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers. Bez podanego klucza
-- NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze są dodatnie.
CREATE TRIGGER stanowisko_kontakty_wpisy_bez_zastepowania
BEFORE INSERT ON stanowisko_kontakty_wpisy
WHEN EXISTS (
  SELECT 1
  FROM stanowisko_kontakty_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- R9: stary kontakt zostaje jako historia.
CREATE TRIGGER stanowisko_kontakty_wpisy_bez_usuwania
BEFORE DELETE ON stanowisko_kontakty_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;
