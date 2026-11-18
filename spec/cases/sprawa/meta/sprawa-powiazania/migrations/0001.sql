-- Odnośniki do innych teczek i pokoi, np. z teczki transakcji do teczki oferty sprzedającego albo
-- do pokoju wymiany z kancelarią. Inny pokój wskazuje się tekstem, bo klucz obcy działa tylko
-- w obrębie jednej bazy. Odnośnik nie daje dostępu do tamtej teczki (R1).
--
-- Wpisuje je agent albo kierownik, który zakłada związek. Powiązanie jest zapisem tego, kto
-- i kiedy połączył sprawy, więc się go nie zmienia.
CREATE TABLE sprawa_powiazania_wpisy (
  -- Klucz nadany przez bazę jest dodatni; zob. sprawa_powiazania_wpisy_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('sprawa_zrodlowa', 'sprawa_wynikowa', 'poprzednia_sprawa_klienta', 'wspolna_umowa', 'pokoj_wymiany', 'inna')),
  -- Typ i oznaczenie tamtej teczki, np. oferta i „Oferta 2026/041”; pokój wymiany ich nie ma.
  typ_teczki TEXT CHECK (typ_teczki <> '' AND typ_teczki NOT GLOB '*[^a-z_]*'),
  oznaczenie TEXT,
  -- Pokój i wiersz, tekstem.
  odnosnik TEXT NOT NULL,
  wpisal TEXT NOT NULL,
  wpisano TEXT NOT NULL
) STRICT;

CREATE TRIGGER sprawa_powiazania_bez_zmian
BEFORE UPDATE ON sprawa_powiazania_wpisy
BEGIN
  SELECT RAISE(ABORT, 'powiązania się nie zmienia; nowe powiązanie to nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER sprawa_powiazania_wpisy_bez_zastepowania
BEFORE INSERT ON sprawa_powiazania_wpisy
WHEN EXISTS (
  SELECT 1
  FROM sprawa_powiazania_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER sprawa_powiazania_wpisy_bez_usuwania
BEFORE DELETE ON sprawa_powiazania_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;
