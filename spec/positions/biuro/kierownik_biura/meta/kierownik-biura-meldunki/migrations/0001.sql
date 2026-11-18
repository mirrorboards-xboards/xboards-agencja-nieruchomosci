-- Meldunki Recepcji i kierowników Pośrednictwa, Transakcji i Najmu. Z nich powstaje przebieg
-- obłożenia w sprawozdaniu, więc kierownik trzyma wszystkie, a nie tylko ostatni. Kontrola nie
-- melduje Kierownikowi biura, bo jej nie podlega.
--
-- Liczby są informacją od nadawcy, a twierdzenia (np. że inny dział opóźnia sprawy) zapisuje się
-- jako jego słowa, które kierownik sprawdza w liczbach. Meldunki tylko się dopisuje.
CREATE TABLE kierownik_biura_meldunki_wpisy (
  -- Klucz nadany przez bazę jest dodatni; zob. kierownik_biura_meldunki_wpisy_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  nadawca TEXT NOT NULL CHECK (nadawca IN ('recepcja', 'posrednictwo', 'transakcje', 'najem')),
  -- Tożsamość agenta, który meldunek wysłał.
  tozsamosc TEXT NOT NULL CHECK (tozsamosc <> ''),
  otrzymano TEXT NOT NULL,
  -- Pola meldunku kierownika działu.
  agenci_pracuja INTEGER CHECK (agenci_pracuja >= 0),
  sprawy_zablokowane INTEGER CHECK (sprawy_zablokowane >= 0),
  decyzje_czekaja INTEGER CHECK (decyzje_czekaja >= 0),
  najblizszy_termin TEXT,
  -- Pola meldunku Recepcji.
  wiadomosci_wplynely INTEGER CHECK (wiadomosci_wplynely >= 0),
  karty_wyslane INTEGER CHECK (karty_wyslane >= 0),
  zaklocenia INTEGER CHECK (zaklocenia >= 0),
  -- To, co meldunek mówi poza liczbami, jako słowa nadawcy.
  twierdzenia TEXT,

  CONSTRAINT pola_meldunku_dzialu CHECK (
    (nadawca <> 'recepcja') = (agenci_pracuja IS NOT NULL)
    AND (nadawca <> 'recepcja') = (sprawy_zablokowane IS NOT NULL)
    AND (nadawca <> 'recepcja') = (decyzje_czekaja IS NOT NULL)
    AND (nadawca <> 'recepcja' OR najblizszy_termin IS NULL)
  ),
  CONSTRAINT pola_meldunku_recepcji CHECK (
    (nadawca = 'recepcja') = (wiadomosci_wplynely IS NOT NULL)
    AND (nadawca = 'recepcja') = (karty_wyslane IS NOT NULL)
    AND (nadawca = 'recepcja') = (zaklocenia IS NOT NULL)
  )
) STRICT;

CREATE TRIGGER kierownik_biura_meldunki_bez_zmian
BEFORE UPDATE ON kierownik_biura_meldunki_wpisy
BEGIN
  SELECT RAISE(ABORT, 'meldunki tylko się dopisuje; zmiany nie są dozwolone');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers. Bez podanego klucza
-- NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze są dodatnie.
CREATE TRIGGER kierownik_biura_meldunki_wpisy_bez_zastepowania
BEFORE INSERT ON kierownik_biura_meldunki_wpisy
WHEN EXISTS (
  SELECT 1
  FROM kierownik_biura_meldunki_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- R9: z dawnych meldunków powstaje przebieg obłożenia.
CREATE TRIGGER kierownik_biura_meldunki_wpisy_bez_usuwania
BEFORE DELETE ON kierownik_biura_meldunki_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); meldunki tylko się dopisuje');
END;
