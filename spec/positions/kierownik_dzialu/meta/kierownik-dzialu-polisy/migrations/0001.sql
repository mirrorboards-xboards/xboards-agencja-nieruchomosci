-- Rozesłanie polisy: które umowy pośrednictwa albo o zarządzanie dostały kopię nowej polisy i kiedy.
-- Biuro niezwłocznie przekazuje klientowi kopię nowej polisy (art. 181 ust. 3a i art. 186 ust. 3a
-- ugn), a kierownik działu poleca to każdemu agentowi z otwartą umową. Wiersz powstaje z raportu
-- agenta, który kopię wysłał.
--
-- Polisę prowadzi Kierownik biura w swojej pamięci; tu jest jej numer i odnośnik do tamtego wpisu.
CREATE TABLE kierownik_dzialu_polisy_rozeslania (
  -- Klucz nadany przez bazę jest dodatni; zob. kierownik_dzialu_polisy_rozeslania_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  polisa_numer TEXT NOT NULL CHECK (polisa_numer <> ''),
  -- Wpis polisy w pamięci Kierownika biura, tekstem.
  polisa_odnosnik TEXT,
  -- Umowa z biurem w teczce, tekstem: pokój i wiersz.
  umowa_odnosnik TEXT NOT NULL CHECK (umowa_odnosnik <> ''),
  raport_id TEXT NOT NULL REFERENCES kierownik_dzialu_raporty_wpisy (id),
  rozeslano TEXT NOT NULL,

  -- Umowa dostaje kopię jednej polisy raz.
  CONSTRAINT polisa_raz_do_umowy UNIQUE (polisa_numer, umowa_odnosnik)
) STRICT;

CREATE TRIGGER kierownik_dzialu_polisy_bez_zmian
BEFORE UPDATE ON kierownik_dzialu_polisy_rozeslania
BEGIN
  SELECT RAISE(ABORT, 'rozesłania polisy się nie zmienia; jest śladem wysłanej kopii');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers; także wiersz tej samej
-- polisy i umowy. Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza,
-- bo klucze są dodatnie.
CREATE TRIGGER kierownik_dzialu_polisy_rozeslania_bez_zastepowania
BEFORE INSERT ON kierownik_dzialu_polisy_rozeslania
WHEN EXISTS (
  SELECT 1
  FROM kierownik_dzialu_polisy_rozeslania
  WHERE id = NEW.id
    OR (polisa_numer = NEW.polisa_numer AND umowa_odnosnik = NEW.umowa_odnosnik)
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- R9: ślad wysłanej kopii zostaje.
CREATE TRIGGER kierownik_dzialu_polisy_rozeslania_bez_usuwania
BEFORE DELETE ON kierownik_dzialu_polisy_rozeslania
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;
