-- Wątpliwości Kontrolerów z powodem, którego nie ma w teczce, i decyzja człowieka, która wątpliwość
-- zdjęła, oraz niemożność zastosowania środka bezpieczeństwa finansowego. Teczka ma tylko znacznik
-- (art. 54 ust. 1 AML); powód i uzasadnienie zdjęcia są wyłącznie tutaj.
--
-- Kierownik kontroli wpisuje zgłoszenie w dniu raportu. Zdjęcie wpisuje się raz: wskazuje
-- zatwierdzoną decyzję zdjecie_watpliwosci osoby wyznaczonej o tej wątpliwości, a decyzja ma
-- uzasadnienie. Niemożność Kierownik wpisuje, gdy zapadła decyzja o nienawiązaniu albo rozwiązaniu
-- stosunków i ocena, czy zawiadomić Generalnego Inspektora.
CREATE TABLE rejestr_kontroli_watpliwosci_wpisy (
  id TEXT PRIMARY KEY,
  -- Teczka i znacznik w niej (sprawa_metryka_watpliwosci); teczka ma sam znacznik bez powodu.
  teczka_odnosnik TEXT NOT NULL CHECK (teczka_odnosnik <> ''),
  znacznik_odnosnik TEXT NOT NULL CHECK (znacznik_odnosnik <> ''),
  -- Klient, którego wątpliwość dotyczy; NULL przy drugiej stronie, która nie jest klientem.
  klient_id TEXT REFERENCES rejestr_kontroli_klienci_wpisy (id),
  -- Kontroler, który zgłosił wątpliwość, i kiedy.
  zglosil TEXT NOT NULL CHECK (zglosil <> ''),
  zgloszono TEXT NOT NULL,
  powod TEXT NOT NULL CHECK (powod <> ''),
  raport_odnosnik TEXT NOT NULL CHECK (raport_odnosnik <> ''),
  wpisal TEXT NOT NULL,
  wpisano TEXT NOT NULL,
  -- Decyzja zdjecie_watpliwosci osoby wyznaczonej z uzasadnieniem; NULL znaczy, że wątpliwość stoi.
  zdjecie_decyzja_id TEXT REFERENCES rejestr_kontroli_decyzje_wpisy (id),

  UNIQUE (teczka_odnosnik, znacznik_odnosnik)
) STRICT;

CREATE TABLE rejestr_kontroli_watpliwosci_niemoznosci (
  -- Klucz nadany przez bazę jest dodatni; zob. rejestr_kontroli_watpliwosci_niemoznosci_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  klient_id TEXT NOT NULL REFERENCES rejestr_kontroli_klienci_wpisy (id),
  -- Środek z art. 34 ust. 1 AML, którego nie dało się zastosować.
  srodek TEXT NOT NULL CHECK (srodek IN ('identyfikacja', 'weryfikacja', 'beneficjent', 'cel_i_charakter', 'monitorowanie')),
  opis TEXT NOT NULL CHECK (opis <> ''),
  -- Decyzja o nienawiązaniu albo rozwiązaniu stosunków (art. 41 ust. 1 AML).
  decyzja_id TEXT NOT NULL REFERENCES rejestr_kontroli_decyzje_wpisy (id),
  -- Czy to podstawa do zawiadomienia Generalnego Inspektora (art. 41 ust. 2 AML) i dlaczego.
  ocena_zawiadomienia TEXT NOT NULL CHECK (ocena_zawiadomienia <> ''),
  raport_odnosnik TEXT NOT NULL CHECK (raport_odnosnik <> ''),
  wpisal TEXT NOT NULL,
  wpisano TEXT NOT NULL
) STRICT;

CREATE TRIGGER rejestr_kontroli_watpliwosci_wpisy_powstaje_niezdjeta
BEFORE INSERT ON rejestr_kontroli_watpliwosci_wpisy
WHEN NEW.zdjecie_decyzja_id IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'wątpliwość powstaje jako stojąca; zdjęcie wpisuje się po decyzji');
END;

CREATE TRIGGER rejestr_kontroli_watpliwosci_wpisy_zdjecie_wskazuje_decyzje
BEFORE UPDATE OF zdjecie_decyzja_id ON rejestr_kontroli_watpliwosci_wpisy
WHEN NEW.zdjecie_decyzja_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM rejestr_kontroli_decyzje_zatwierdzone
    WHERE id = NEW.zdjecie_decyzja_id
      AND rodzaj = 'zdjecie_watpliwosci'
      AND dotyczy_tabela = 'rejestr_kontroli_watpliwosci_wpisy'
      AND dotyczy_id = NEW.id
  )
BEGIN
  SELECT RAISE(ABORT, 'zdjęcie wątpliwości wskazuje zatwierdzoną decyzję zdjecie_watpliwosci o tej wątpliwości');
END;

CREATE TRIGGER rejestr_kontroli_watpliwosci_wpisy_bez_zmian
BEFORE UPDATE ON rejestr_kontroli_watpliwosci_wpisy
WHEN NEW.id IS NOT OLD.id
  OR NEW.teczka_odnosnik IS NOT OLD.teczka_odnosnik
  OR NEW.znacznik_odnosnik IS NOT OLD.znacznik_odnosnik
  OR NEW.klient_id IS NOT OLD.klient_id
  OR NEW.zglosil IS NOT OLD.zglosil
  OR NEW.zgloszono IS NOT OLD.zgloszono
  OR NEW.powod IS NOT OLD.powod
  OR NEW.raport_odnosnik IS NOT OLD.raport_odnosnik
  OR NEW.wpisal IS NOT OLD.wpisal
  OR NEW.wpisano IS NOT OLD.wpisano
  OR (OLD.zdjecie_decyzja_id IS NOT NULL AND NEW.zdjecie_decyzja_id IS NOT OLD.zdjecie_decyzja_id)
BEGIN
  SELECT RAISE(ABORT, 'wątpliwości się nie zmienia; zdjęcie wpisuje się raz');
END;

CREATE TRIGGER rejestr_kontroli_watpliwosci_niemoznosci_po_decyzji_o_nienawiazaniu
BEFORE INSERT ON rejestr_kontroli_watpliwosci_niemoznosci
WHEN NOT EXISTS (
  SELECT 1
  FROM rejestr_kontroli_decyzje_zatwierdzone
  WHERE id = NEW.decyzja_id
    AND rodzaj = 'nienawiazanie_stosunkow'
    AND dotyczy_tabela = 'rejestr_kontroli_klienci_wpisy'
    AND dotyczy_id = NEW.klient_id
)
BEGIN
  SELECT RAISE(ABORT, 'niemożność środka wskazuje zatwierdzoną decyzję nienawiazanie_stosunkow o tym kliencie');
END;

CREATE TRIGGER rejestr_kontroli_watpliwosci_niemoznosci_bez_zmian
BEFORE UPDATE ON rejestr_kontroli_watpliwosci_niemoznosci
BEGIN
  SELECT RAISE(ABORT, 'wpisu niemożności się nie zmienia; błąd poprawia nowy wpis');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers. Bez podanego klucza
-- NEW.id tabel z kluczem liczbowym jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- są dodatnie.
CREATE TRIGGER rejestr_kontroli_watpliwosci_wpisy_bez_zastepowania
BEFORE INSERT ON rejestr_kontroli_watpliwosci_wpisy
WHEN EXISTS (
    SELECT 1
    FROM rejestr_kontroli_watpliwosci_wpisy
    WHERE id = NEW.id
  )
  OR EXISTS (
    SELECT 1
    FROM rejestr_kontroli_watpliwosci_wpisy
    WHERE teczka_odnosnik = NEW.teczka_odnosnik
      AND znacznik_odnosnik = NEW.znacznik_odnosnik
  )
BEGIN
  SELECT RAISE(ABORT, 'wiersza rejestru nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER rejestr_kontroli_watpliwosci_niemoznosci_bez_zastepowania
BEFORE INSERT ON rejestr_kontroli_watpliwosci_niemoznosci
WHEN EXISTS (
  SELECT 1
  FROM rejestr_kontroli_watpliwosci_niemoznosci
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza rejestru nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

-- Art. 49 AML i R9: dokumentację przechowuje się co najmniej 5 lat, a na razie bez terminu.
CREATE TRIGGER rejestr_kontroli_watpliwosci_wpisy_bez_usuwania
BEFORE DELETE ON rejestr_kontroli_watpliwosci_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy rejestru się nie usuwa (R9); błąd poprawia nowy wpis');
END;

CREATE TRIGGER rejestr_kontroli_watpliwosci_niemoznosci_bez_usuwania
BEFORE DELETE ON rejestr_kontroli_watpliwosci_niemoznosci
BEGIN
  SELECT RAISE(ABORT, 'wierszy rejestru się nie usuwa (R9); błąd poprawia nowy wpis');
END;
