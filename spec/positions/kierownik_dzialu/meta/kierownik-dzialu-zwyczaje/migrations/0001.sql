-- Zwyczaje kontrahentów: jak portal, kanał sprzedaży, kancelaria, bank, spółdzielnia, wspólnota,
-- urząd, wykonawca albo deweloper zwykle postępuje, z tematem, opisem, źródłem, pewnością i datą.
-- To ustalenia kierownika o kontrahentach; polecenia leżą w stanowisko-polecenia.
--
-- Źródło zwyczaju to wiersz w stanowisko-zrodla, bo gdy źródło poda nieprawdę, kierownik ocenia na nowo
-- wszystkie zwyczaje od niego (operacja zwyczaje_od_zrodla). Raport albo wiadomość, z której zwyczaj
-- pochodzi, wskazuje odnośnik.
--
-- Zwyczaj jest pewny tylko z potwierdzeniem: regulaminem, potwierdzeniem kontrahenta albo podpisem
-- człowieka biura. Podpis przychodzi w wiadomości, a kierownik przenosi go do wiersza; sprawdza go
-- aplikacja. Zwyczaj, który okazał się nieprawdziwy, traci pewność albo zostaje wykreślony;
-- wykreślony zostaje w pamięci jako ślad.
CREATE TABLE kierownik_dzialu_zwyczaje_wpisy (
  -- Klucz nadany przez bazę jest dodatni; zob. kierownik_dzialu_zwyczaje_wpisy_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  kontrahent_rodzaj TEXT NOT NULL CHECK (kontrahent_rodzaj IN ('portal', 'kanal_sprzedazy', 'kancelaria', 'bank', 'spoldzielnia', 'wspolnota', 'urzad', 'wykonawca', 'deweloper')),
  -- Np. „Spółdzielnia Mieszkaniowa Leśna” albo nazwa portalu.
  kontrahent TEXT NOT NULL CHECK (kontrahent <> ''),
  temat TEXT NOT NULL,
  opis TEXT NOT NULL,
  zrodlo_id TEXT REFERENCES stanowisko_zrodla_wpisy (id),
  -- Raport, wiadomość albo wiersz teczki, z którego zwyczaj pochodzi, tekstem.
  zrodlo_odnosnik TEXT,
  ustalono TEXT NOT NULL,
  pewnosc TEXT NOT NULL CHECK (pewnosc IN ('pewny', 'prawdopodobny', 'niepewny')),
  potwierdzenie TEXT CHECK (potwierdzenie IN ('regulamin', 'kontrahent', 'czlowiek_biura')),
  -- Regulamin albo wiadomość kontrahenta, tekstem.
  potwierdzenie_odnosnik TEXT,
  potwierdzil TEXT,
  podpis TEXT,
  potwierdzono TEXT,
  wykreslono TEXT,
  powod_wykreslenia TEXT,

  CONSTRAINT zwyczaj_ze_zrodlem CHECK (zrodlo_id IS NOT NULL OR zrodlo_odnosnik IS NOT NULL),
  CONSTRAINT pewny_z_potwierdzeniem CHECK ((pewnosc = 'pewny') = (potwierdzenie IS NOT NULL)),
  CONSTRAINT potwierdzenie_z_data CHECK ((potwierdzenie IS NULL) = (potwierdzono IS NULL)),
  CONSTRAINT potwierdzenie_regulaminem_albo_kontrahenta_wskazane CHECK (
    (potwierdzenie IS NOT NULL AND potwierdzenie IN ('regulamin', 'kontrahent')) = (potwierdzenie_odnosnik IS NOT NULL)
  ),
  CONSTRAINT potwierdzenie_czlowieka_biura_z_podpisem CHECK (
    (potwierdzenie IS 'czlowiek_biura') = (potwierdzil IS NOT NULL)
    AND (potwierdzil IS NULL) = (podpis IS NULL)
  ),
  CONSTRAINT wykreslenie_z_powodem CHECK ((wykreslono IS NULL) = (powod_wykreslenia IS NULL))
) STRICT;

-- Potwierdzenie i wykreślenie przychodzą później, więc zwyczaj powstaje bez nich, a przez to nie
-- jest jeszcze pewny.
CREATE TRIGGER kierownik_dzialu_zwyczaje_powstaje_niepotwierdzony
BEFORE INSERT ON kierownik_dzialu_zwyczaje_wpisy
WHEN NEW.potwierdzenie IS NOT NULL
  OR NEW.wykreslono IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'zwyczaj powstaje niepotwierdzony i niewykreślony');
END;

CREATE TRIGGER kierownik_dzialu_zwyczaje_wykreslony_bez_zmian
BEFORE UPDATE ON kierownik_dzialu_zwyczaje_wpisy
WHEN OLD.wykreslono IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'wykreślonego zwyczaju się nie zmienia');
END;

-- Na tym, kogo zwyczaj dotyczy i skąd pochodzi, stoi ponowna ocena od źródła; inny zwyczaj to nowy
-- wpis.
CREATE TRIGGER kierownik_dzialu_zwyczaje_tresc_niezmienna
BEFORE UPDATE ON kierownik_dzialu_zwyczaje_wpisy
WHEN NEW.id IS NOT OLD.id
  OR NEW.kontrahent_rodzaj IS NOT OLD.kontrahent_rodzaj
  OR NEW.kontrahent IS NOT OLD.kontrahent
  OR NEW.temat IS NOT OLD.temat
  OR NEW.opis IS NOT OLD.opis
  OR NEW.zrodlo_id IS NOT OLD.zrodlo_id
  OR NEW.zrodlo_odnosnik IS NOT OLD.zrodlo_odnosnik
  OR NEW.ustalono IS NOT OLD.ustalono
BEGIN
  SELECT RAISE(ABORT, 'kontrahenta, treści, źródła ani daty zwyczaju się nie zmienia; inny zwyczaj to nowy wpis');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers. Bez podanego klucza
-- NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze są dodatnie.
CREATE TRIGGER kierownik_dzialu_zwyczaje_wpisy_bez_zastepowania
BEFORE INSERT ON kierownik_dzialu_zwyczaje_wpisy
WHEN EXISTS (
  SELECT 1
  FROM kierownik_dzialu_zwyczaje_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- R9: zwyczaj, który przestał obowiązywać, wykreśla się, a nie usuwa.
CREATE TRIGGER kierownik_dzialu_zwyczaje_wpisy_bez_usuwania
BEFORE DELETE ON kierownik_dzialu_zwyczaje_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;
