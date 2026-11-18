-- Nieobecności i zastępcy: kto jest nieobecny, od kiedy do kiedy i kto dostaje w tym czasie jego
-- listę decyzji, z odwołaniem do wiadomości, którą to zgłoszono.
--
-- Kierownik przyjmuje nieobecność tylko z podpisem człowieka, którego dotyczy, albo właściciela
-- biura, bo niepodpisana wiadomość mogłaby przekierować czyjąś listę (podszywanie się). Podpis
-- sprawdza aplikacja, a baza wymaga jego obecności. Odwołanie też przychodzi podpisane.
CREATE TABLE kierownik_biura_nieobecnosci_wpisy (
  -- Klucz nadany przez bazę jest dodatni; zob. kierownik_biura_nieobecnosci_wpisy_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  -- Tożsamość człowieka, który będzie nieobecny.
  czlowiek TEXT NOT NULL CHECK (czlowiek <> ''),
  -- Daty RRRR-MM-DD, obie włącznie.
  od_dnia TEXT NOT NULL,
  do_dnia TEXT NOT NULL,
  -- Tożsamość człowieka, który dostaje listę.
  zastepca TEXT NOT NULL CHECK (zastepca <> ''),
  podpisal TEXT NOT NULL,
  podpisal_jako TEXT NOT NULL CHECK (podpisal_jako IN ('czlowiek', 'wlasciciel_biura')),
  podpis TEXT NOT NULL CHECK (podpis <> ''),
  -- Wiadomość, którą zgłoszono nieobecność, tekstem.
  wiadomosc_odnosnik TEXT NOT NULL CHECK (wiadomosc_odnosnik <> ''),
  zapisano TEXT NOT NULL,
  odwolal TEXT,
  odwolanie_podpis TEXT,
  odwolano TEXT,

  CONSTRAINT nieobecnosc_od_przed_do CHECK (od_dnia <= do_dnia),
  CONSTRAINT zastepca_to_ktos_inny CHECK (zastepca <> czlowiek),
  CONSTRAINT podpis_czlowieka_ktorego_dotyczy CHECK (podpisal_jako <> 'czlowiek' OR podpisal = czlowiek),
  CONSTRAINT odwolanie_z_podpisem CHECK (
    (odwolano IS NULL) = (odwolal IS NULL)
    AND (odwolano IS NULL) = (odwolanie_podpis IS NULL)
  )
) STRICT;

CREATE TRIGGER kierownik_biura_nieobecnosci_powstaje_nieodwolana
BEFORE INSERT ON kierownik_biura_nieobecnosci_wpisy
WHEN NEW.odwolano IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'nieobecność powstaje nieodwołana; odwołanie przychodzi później');
END;

-- Podpis obejmuje nieobecność, więc zmienia się tylko raz odnotowane odwołanie; inna nieobecność
-- to nowy wpis.
CREATE TRIGGER kierownik_biura_nieobecnosci_bez_zmian
BEFORE UPDATE ON kierownik_biura_nieobecnosci_wpisy
WHEN OLD.odwolano IS NOT NULL
  OR NEW.id IS NOT OLD.id
  OR NEW.czlowiek IS NOT OLD.czlowiek
  OR NEW.od_dnia IS NOT OLD.od_dnia
  OR NEW.do_dnia IS NOT OLD.do_dnia
  OR NEW.zastepca IS NOT OLD.zastepca
  OR NEW.podpisal IS NOT OLD.podpisal
  OR NEW.podpisal_jako IS NOT OLD.podpisal_jako
  OR NEW.podpis IS NOT OLD.podpis
  OR NEW.wiadomosc_odnosnik IS NOT OLD.wiadomosc_odnosnik
  OR NEW.zapisano IS NOT OLD.zapisano
BEGIN
  SELECT RAISE(ABORT, 'nieobecności się nie zmienia; wolno ją raz odwołać, a inna nieobecność to nowy wpis');
END;

-- Odwołuje ten sam człowiek albo właściciel biura, bo inaczej odwołanie byłoby podszyciem.
CREATE TRIGGER kierownik_biura_nieobecnosci_odwoluje_czlowiek_albo_wlasciciel_biura
BEFORE UPDATE OF odwolal ON kierownik_biura_nieobecnosci_wpisy
WHEN NEW.odwolal IS NOT NULL
  AND NEW.odwolal IS NOT NEW.czlowiek
  AND NOT (NEW.podpisal_jako = 'wlasciciel_biura' AND NEW.odwolal = NEW.podpisal)
BEGIN
  SELECT RAISE(ABORT, 'nieobecność odwołuje człowiek, którego dotyczy, albo właściciel biura, który ją zgłosił');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers. Bez podanego klucza
-- NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze są dodatnie.
CREATE TRIGGER kierownik_biura_nieobecnosci_wpisy_bez_zastepowania
BEFORE INSERT ON kierownik_biura_nieobecnosci_wpisy
WHEN EXISTS (
  SELECT 1
  FROM kierownik_biura_nieobecnosci_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- R9: dawne nieobecności zostają w sprawozdaniach.
CREATE TRIGGER kierownik_biura_nieobecnosci_wpisy_bez_usuwania
BEFORE DELETE ON kierownik_biura_nieobecnosci_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;
