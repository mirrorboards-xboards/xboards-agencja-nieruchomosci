-- Metryka teczki: jeden wiersz na teczkę, dziennik etapów i znaczniki wątpliwości Kontrolerów.
-- Metrykę zakłada człowiek, który przyjmuje sprawę i jest właścicielem teczki. Etap zmienia agent
-- sprawy, a znacznik stawia Kontroler. Dział nie ma kolumny, bo wynika z typu teczki.
--
-- Rdzeń zna tylko etapy przyjeta, przerwana i zamknieta. Pozostałe etapy i dozwolone przejścia
-- daje typ sprawy w swoim rodzaju <typ>-reguly.
--
-- Przerwanie i wznowienie wymagają zatwierdzonej decyzji człowieka. Po przerwaniu sprawa się
-- zamyka albo wraca do etapu, w którym była przed przerwaniem; przerwać i wznowić można ją więcej
-- niż raz. Etap, do którego wolno wrócić, mówi widok sprawa_metryka_powrot_z_przerwania; typy
-- dopuszczają ten powrót w swoich przejściach według niego.
--
-- Powód wątpliwości nie ma tu kolumny: zostaje w rejestrze kontroli, bo teczkę czytają osoby
-- obsługujące klienta (art. 54 ust. 1 AML).
--
-- Reguły, które zależą od późniejszych rodzajów, leżą w ich migracjach:
-- - znacznik blokuje umowę do podpisu: sprawa-umowy (sprawa_umowy_watpliwosc_blokuje_podpis);
-- - teczki nie zamyka się przy przygotowanej wypłacie: sprawa-wyplaty
--   (sprawa_wyplaty_zamkniecie_po_wyplatach).
CREATE TABLE sprawa_metryka_wpis (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  -- Np. „Oferta 2026/041, ul. Polna 7 m. 12”.
  oznaczenie TEXT NOT NULL,
  -- Nazwa folderu typu sprawy z cases/. Bez listy wartości, bo nowy typ sprawy nie może wymagać
  -- migracji rdzenia, wspólnej dla wszystkich typów.
  typ_pokoju TEXT NOT NULL CHECK (typ_pokoju <> '' AND typ_pokoju NOT GLOB '*[^a-z_]*'),
  przyjal TEXT NOT NULL,
  przyjeto TEXT NOT NULL
) STRICT;

-- Dziennik wejść w etapy: profile wymagają daty wejścia w każdy etap. Bieżący etap to ostatni
-- wiersz.
CREATE TABLE sprawa_metryka_etapy (
  -- Klucz nadany przez bazę jest dodatni; zob. sprawa_metryka_etapy_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  -- Nazwa etapu: przyjeta, przerwana, zamknieta albo etap typu sprawy.
  etap TEXT NOT NULL CHECK (etap <> '' AND etap NOT GLOB '*[^a-z_]*'),
  od TEXT NOT NULL,
  zmienil TEXT NOT NULL,
  -- Decyzja, na której stoi wejście w etap; wymagana przy przerwaniu i wznowieniu sprawy.
  decyzja_id TEXT REFERENCES sprawa_decyzje_wpisy (id)
) STRICT;

-- Znacznik wątpliwości: każdy Kontroler stawia własny, bez powodu. Rejestr kontroli wskazuje go
-- identyfikatorem, więc ma UUID.
CREATE TABLE sprawa_metryka_watpliwosci (
  id TEXT PRIMARY KEY,
  postawil TEXT NOT NULL,
  postawiono TEXT NOT NULL,
  -- Decyzja zdjecie_watpliwosci, która zdjęła znacznik; NULL znaczy, że znacznik stoi.
  zdjecie_decyzja_id TEXT REFERENCES sprawa_decyzje_wpisy (id)
) STRICT;

-- Metryka mówi, jakiego typu jest teczka, kto ją przyjął i kiedy oraz pod jakim oznaczeniem
-- wskazują ją powiązania innych teczek. Człowiek, który przyjął sprawę, jest właścicielem teczki,
-- więc żadnej z tych danych się nie zmienia.
CREATE TRIGGER sprawa_metryka_wpis_niezmienny
BEFORE UPDATE ON sprawa_metryka_wpis
BEGIN
  SELECT RAISE(ABORT, 'metryki teczki się nie zmienia: typ pokoju, oznaczenie, kto i kiedy przyjął sprawę');
END;

CREATE TRIGGER sprawa_metryka_pierwszy_etap_to_przyjeta
BEFORE INSERT ON sprawa_metryka_etapy
WHEN NEW.etap <> 'przyjeta'
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_metryka_etapy
  )
BEGIN
  SELECT RAISE(ABORT, 'pierwszy etap sprawy to przyjeta');
END;

CREATE TRIGGER sprawa_metryka_po_zamknieciu_bez_etapow
BEFORE INSERT ON sprawa_metryka_etapy
WHEN EXISTS (
  SELECT 1
  FROM sprawa_metryka_etapy
  WHERE etap = 'zamknieta'
)
BEGIN
  SELECT RAISE(ABORT, 'po etapie zamknieta sprawa nie ma kolejnych etapów');
END;

-- Gdy bieżący etap to przerwana: etap bezpośrednio przed nim, do którego wraca wznowiona sprawa,
-- i chwila przerwania. W innym etapie widok jest pusty.
CREATE VIEW sprawa_metryka_powrot_z_przerwania AS
SELECT
  przed.etap AS etap,
  przerwanie.od AS przerwano
FROM sprawa_metryka_etapy AS przerwanie
JOIN sprawa_metryka_etapy AS przed ON przed.id = (
  SELECT max(id)
  FROM sprawa_metryka_etapy
  WHERE id < przerwanie.id
)
WHERE przerwanie.id = (
    SELECT max(id)
    FROM sprawa_metryka_etapy
  )
  AND przerwanie.etap = 'przerwana';

-- Przerwana sprawa się zamyka albo wraca do etapu sprzed przerwania (T1). Do innego etapu nie
-- przechodzi, nawet gdy typ by na to pozwalał.
CREATE TRIGGER sprawa_metryka_po_przerwaniu_zamkniecie_albo_powrot
BEFORE INSERT ON sprawa_metryka_etapy
WHEN NEW.etap <> 'zamknieta'
  AND (
    SELECT etap = 'przerwana'
    FROM sprawa_metryka_etapy
    ORDER BY id DESC
    LIMIT 1
  )
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_metryka_powrot_z_przerwania
    WHERE etap = NEW.etap
  )
BEGIN
  SELECT RAISE(ABORT, 'po etapie przerwana przychodzi zamknieta albo powrót do etapu sprzed przerwania');
END;

-- Wznowienie przerwanej sprawy wymaga zgody człowieka wydanej po przerwaniu: zgoda sprzed
-- przerwania dotyczyła innej sytuacji.
CREATE TRIGGER sprawa_metryka_wznowienie_wymaga_decyzji
BEFORE INSERT ON sprawa_metryka_etapy
WHEN NEW.etap <> 'zamknieta'
  AND EXISTS (
    SELECT 1
    FROM sprawa_metryka_powrot_z_przerwania
  )
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_decyzje_zatwierdzone AS decyzja
    JOIN sprawa_metryka_powrot_z_przerwania AS powrot
    WHERE decyzja.id = NEW.decyzja_id
      AND decyzja.rodzaj = 'wznowienie_sprawy'
      AND decyzja.podpisano >= powrot.przerwano
  )
BEGIN
  SELECT RAISE(ABORT, 'powrót z etapu przerwana wymaga zatwierdzonej decyzji wznowienie_sprawy, która zapadła po przerwaniu');
END;

-- Jedna zgoda człowieka pozwala na jedno przerwanie albo jedno wznowienie. Inaczej po kolejnym
-- przerwaniu stara decyzja wznowiłaby sprawę bez nowej zgody.
CREATE TRIGGER sprawa_metryka_decyzja_etapu_raz
BEFORE INSERT ON sprawa_metryka_etapy
WHEN NEW.decyzja_id IS NOT NULL
  AND EXISTS (
    SELECT 1
    FROM sprawa_metryka_etapy
    WHERE decyzja_id = NEW.decyzja_id
  )
BEGIN
  SELECT RAISE(ABORT, 'jedna decyzja stoi za jednym wejściem w etap; kolejne przerwanie albo wznowienie wymaga nowej decyzji');
END;

CREATE TRIGGER sprawa_metryka_przerwanie_wymaga_decyzji
BEFORE INSERT ON sprawa_metryka_etapy
WHEN NEW.etap = 'przerwana'
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_decyzje_zatwierdzone AS decyzja
    WHERE decyzja.id = NEW.decyzja_id
      AND decyzja.rodzaj = 'przerwanie_sprawy'
  )
BEGIN
  SELECT RAISE(ABORT, 'etap przerwana wymaga zatwierdzonej decyzji przerwanie_sprawy');
END;

-- Dziennik etapów jest zapisem tego, kto i kiedy przesunął sprawę; zmiana wiersza ominęłaby
-- reguły etapów.
CREATE TRIGGER sprawa_metryka_etapy_bez_zmian
BEFORE UPDATE ON sprawa_metryka_etapy
BEGIN
  SELECT RAISE(ABORT, 'wpisu w dzienniku etapów się nie zmienia; nowy etap to nowy wiersz');
END;

-- Znacznik powstaje stojący: zdjęcie sprawdza zmianę, więc nie może przyjść razem z wpisem.
CREATE TRIGGER sprawa_metryka_watpliwosc_powstaje_stojaca
BEFORE INSERT ON sprawa_metryka_watpliwosci
WHEN NEW.zdjecie_decyzja_id IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'znacznik wątpliwości powstaje stojący; zdejmuje go później decyzja');
END;

-- Zdjąć znacznik może tylko człowiek, a decyzja musi wskazywać ten znacznik i zapaść po nim:
-- decyzja o zdjęciu wcześniejszego znacznika nie zdejmuje nowego.
CREATE TRIGGER sprawa_metryka_zdjecie_watpliwosci_wymaga_decyzji
BEFORE UPDATE OF zdjecie_decyzja_id ON sprawa_metryka_watpliwosci
WHEN NEW.zdjecie_decyzja_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_decyzje_zatwierdzone AS decyzja
    WHERE decyzja.id = NEW.zdjecie_decyzja_id
      AND decyzja.rodzaj = 'zdjecie_watpliwosci'
      AND decyzja.dotyczy_tabela = 'sprawa_metryka_watpliwosci'
      AND decyzja.dotyczy_id = NEW.id
      AND decyzja.podpisano >= OLD.postawiono
  )
BEGIN
  SELECT RAISE(ABORT, 'znacznik zdejmuje tylko zatwierdzona decyzja zdjecie_watpliwosci, która go wskazuje i zapadła po nim');
END;

CREATE TRIGGER sprawa_metryka_watpliwosc_zmienia_tylko_zdjecie
BEFORE UPDATE ON sprawa_metryka_watpliwosci
WHEN OLD.zdjecie_decyzja_id IS NOT NULL
  OR NEW.id IS NOT OLD.id
  OR NEW.postawil IS NOT OLD.postawil
  OR NEW.postawiono IS NOT OLD.postawiono
BEGIN
  SELECT RAISE(ABORT, 'znacznika wątpliwości się nie zmienia; zmienia go tylko zdjęcie');
END;

-- Dopóki stoi jakikolwiek znacznik, człowiek nie zgadza się na termin, rachunek ani wypłatę.
-- Odmówić wolno zawsze, a dokumenty agent zbiera dalej.
CREATE TRIGGER sprawa_metryka_watpliwosc_blokuje_zgode
BEFORE UPDATE OF stan ON sprawa_decyzje_wpisy
WHEN NEW.stan = 'rozstrzygnieta'
  AND NEW.rozstrzygniecie IN ('zgoda', 'zgoda_z_warunkiem')
  AND NEW.rodzaj IN ('potwierdzenie_terminu', 'dopuszczenie_rachunku', 'wyplata')
  AND EXISTS (
    SELECT 1
    FROM sprawa_metryka_watpliwosci
    WHERE zdjecie_decyzja_id IS NULL
  )
BEGIN
  SELECT RAISE(ABORT, 'dopóki stoi znacznik wątpliwości, nie ma zgody na termin, rachunek ani wypłatę');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER sprawa_metryka_wpis_bez_zastepowania
BEFORE INSERT ON sprawa_metryka_wpis
WHEN EXISTS (
  SELECT 1
  FROM sprawa_metryka_wpis
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER sprawa_metryka_wpis_bez_usuwania
BEFORE DELETE ON sprawa_metryka_wpis
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER sprawa_metryka_etapy_bez_zastepowania
BEFORE INSERT ON sprawa_metryka_etapy
WHEN EXISTS (
  SELECT 1
  FROM sprawa_metryka_etapy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER sprawa_metryka_etapy_bez_usuwania
BEFORE DELETE ON sprawa_metryka_etapy
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER sprawa_metryka_watpliwosci_bez_zastepowania
BEFORE INSERT ON sprawa_metryka_watpliwosci
WHEN EXISTS (
  SELECT 1
  FROM sprawa_metryka_watpliwosci
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER sprawa_metryka_watpliwosci_bez_usuwania
BEFORE DELETE ON sprawa_metryka_watpliwosci
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;
