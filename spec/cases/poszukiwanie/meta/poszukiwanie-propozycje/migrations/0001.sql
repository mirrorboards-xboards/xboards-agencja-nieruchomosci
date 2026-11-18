-- Propozycje ofert dla klienta, prezentacje i pytania otwarte do strony, która sprzedaje albo
-- wynajmuje. Pisze Opiekun poszukującego. Kwoty w groszach.
--
-- Propozycja to oferta z portalu, z katalogu ofert biura albo od innego biura, porównana
-- z bieżącą potwierdzoną wersją profilu. Dane z ogłoszenia i z katalogu są twierdzeniem strony,
-- która sprzedaje albo wynajmuje. Ofertę z katalogu teczka wskazuje tekstem (inny pokój): wpis
-- i datę jego wersji. Ofertę niepubliczną Opiekun pokazuje tylko klientowi w granicach zgody
-- z katalogu; granice porównuje aplikacja. Przedstawienie propozycji klientowi to praca według
-- umowy pośrednictwa, więc nie jest wcześniejsze niż sprawa_umowy_praca_od.
--
-- Wynik przeglądania księgi Opiekun zapisuje bez oceny skutków; ocenia je Analityk stanu prawnego
-- w transakcji. Danych klienta druga strona dostaje tylko tyle, ile wymaga umówienie prezentacji;
-- szerszy zakres wymaga zatwierdzonej decyzji „inna”, która wskazuje prezentację.

CREATE TABLE poszukiwanie_propozycje_wpisy (
  id TEXT PRIMARY KEY,
  -- Wersja profilu, z którą Opiekun porównał ofertę.
  profil_wersja_id TEXT NOT NULL REFERENCES poszukiwanie_profile_wersje (id),

  -- Skąd oferta: kopia ogłoszenia z portalem i numerem, którą widział klient, albo wpis
  -- z katalogu ofert biura z datą wersji, i kto ją prowadzi.
  zrodlo TEXT NOT NULL CHECK (zrodlo IN ('portal', 'katalog_biura', 'inne_biuro')),
  ogloszenie_sciezka TEXT,
  ogloszenie_skrot TEXT,
  portal TEXT,
  numer TEXT,
  katalog_odnosnik TEXT,
  katalog_wersja_od TEXT,
  prowadzi TEXT NOT NULL CHECK (prowadzi IN ('wlasciciel', 'inne_biuro', 'opiekun_oferty')),
  niepubliczna INTEGER NOT NULL CHECK (niepubliczna IN (0, 1)),

  -- Ocena: warunki spełnione, życzenia niespełnione i rozbieżności zauważone jako fakt, np.
  -- ogłoszenie wróciło pod nowym numerem z innym metrażem.
  zgodnosc TEXT NOT NULL,
  niespelnione_zyczenia TEXT,
  rozbieznosci TEXT,

  -- Księga i cena: wynik przeglądania księgi, gdy Opiekun zna numer, cena z ogłoszenia i ceny
  -- podobnych transakcji z rejestru cen nieruchomości (art. 181a pkt 6 ugn) jako dokument.
  numer_kw TEXT CHECK (numer_kw GLOB '[A-Z][A-Z][0-9][A-Z]/[0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9]/[0-9]'),
  kw_przegladano TEXT,
  kw_ogloszajacy_wpisany TEXT CHECK (kw_ogloszajacy_wpisany IN ('tak', 'nie', 'nieznane')),
  kw_wpisy_iii_iv TEXT,
  cena_ogloszenia INTEGER NOT NULL CHECK (cena_ogloszenia > 0),
  ceny_rcn_dokument_id TEXT REFERENCES sprawa_dokumenty_wpisy (id),
  utworzono TEXT NOT NULL,

  -- Klient: kiedy dostał propozycję i co odpowiedział.
  przedstawiono TEXT,
  reakcja TEXT CHECK (reakcja IN ('chce_obejrzec', 'odrzuca', 'czeka')),
  reakcja_powod TEXT,

  CONSTRAINT z_portalu_z_kopia_i_numerem CHECK (
    zrodlo <> 'portal' OR (portal IS NOT NULL AND numer IS NOT NULL AND ogloszenie_sciezka IS NOT NULL)
  ),
  CONSTRAINT kopia_ze_skrotem CHECK ((ogloszenie_sciezka IS NULL) = (ogloszenie_skrot IS NULL)),
  CONSTRAINT z_katalogu_z_odnosnikiem_i_wersja CHECK (
    (zrodlo = 'katalog_biura') = (katalog_odnosnik IS NOT NULL)
    AND (zrodlo = 'katalog_biura') = (katalog_wersja_od IS NOT NULL)
  ),
  -- Ofertę biura prowadzi Opiekun oferty i tylko ona bywa niepubliczna.
  CONSTRAINT z_katalogu_prowadzi_opiekun_oferty CHECK ((zrodlo = 'katalog_biura') = (prowadzi = 'opiekun_oferty')),
  CONSTRAINT niepubliczna_tylko_z_katalogu CHECK (niepubliczna = 0 OR zrodlo = 'katalog_biura'),
  CONSTRAINT przegladanie_ksiegi_z_numerem_i_wynikiem CHECK (
    (kw_przegladano IS NULL) = (kw_ogloszajacy_wpisany IS NULL)
    AND (kw_przegladano IS NULL OR numer_kw IS NOT NULL)
    AND (kw_wpisy_iii_iv IS NULL OR kw_przegladano IS NOT NULL)
  ),
  CONSTRAINT przedstawiono_po_utworzeniu CHECK (przedstawiono IS NULL OR przedstawiono >= utworzono),
  CONSTRAINT reakcja_po_przedstawieniu CHECK (reakcja IS NULL OR przedstawiono IS NOT NULL),
  CONSTRAINT powod_przy_reakcji CHECK (reakcja_powod IS NULL OR reakcja IS NOT NULL)
) STRICT;

-- Prezentacja oglądanej nieruchomości z opinią klienta i notatką pośrednika.
CREATE TABLE poszukiwanie_propozycje_prezentacje (
  id TEXT PRIMARY KEY,
  propozycja_id TEXT NOT NULL REFERENCES poszukiwanie_propozycje_wpisy (id),
  termin TEXT NOT NULL,
  kto_pokazuje TEXT NOT NULL CHECK (kto_pokazuje IN ('wlasciciel', 'inne_biuro', 'posrednik_biura')),
  -- Dane klienta podane drugiej stronie, np. „imię, nazwisko, telefon”.
  dane_klienta_podane TEXT NOT NULL,
  szerszy_zakres INTEGER NOT NULL CHECK (szerszy_zakres IN (0, 1)),
  szerszy_zakres_decyzja_id TEXT REFERENCES sprawa_decyzje_wpisy (id),
  -- Kto przyjdzie po stronie klienta i czy przyjdzie pośrednik.
  uczestnicy TEXT,
  opinia_klienta TEXT,
  notatka_posrednika TEXT,
  odwolano TEXT,
  odwolal TEXT,
  powod_odwolania TEXT,

  CONSTRAINT szerszy_zakres_z_decyzja CHECK ((szerszy_zakres = 1) = (szerszy_zakres_decyzja_id IS NOT NULL)),
  CONSTRAINT odwolanie_kto_kiedy_dlaczego CHECK (
    (odwolano IS NULL) = (odwolal IS NULL)
    AND (odwolano IS NULL) = (powod_odwolania IS NULL)
  ),
  CONSTRAINT odwolana_bez_opinii_i_notatki CHECK (
    odwolano IS NULL OR (opinia_klienta IS NULL AND notatka_posrednika IS NULL)
  )
) STRICT;

-- Pytania klienta do strony, która pokazuje nieruchomość, i jej odpowiedzi, gdy przyjdą.
CREATE TABLE poszukiwanie_propozycje_pytania (
  -- Klucz nadany przez bazę jest dodatni; zob. poszukiwanie_propozycje_pytania_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  prezentacja_id TEXT NOT NULL REFERENCES poszukiwanie_propozycje_prezentacje (id),
  pytanie TEXT NOT NULL,
  -- Odpowiedź jest twierdzeniem drugiej strony z wpływem, w którym przyszła.
  odpowiedz TEXT,
  odpowiedz_wplyw_id TEXT REFERENCES sprawa_korespondencja_wplywy (id),

  CONSTRAINT odpowiedz_z_wplywem CHECK ((odpowiedz IS NULL) = (odpowiedz_wplyw_id IS NULL))
) STRICT;

-- Propozycję porównuje się z bieżącą wersją profilu, którą klient potwierdził.
CREATE TRIGGER poszukiwanie_propozycje_z_biezacym_profilem
BEFORE INSERT ON poszukiwanie_propozycje_wpisy
WHEN NOT EXISTS (
  SELECT 1
  FROM poszukiwanie_profile_biezace
  WHERE id = NEW.profil_wersja_id
)
BEGIN
  SELECT RAISE(ABORT, 'propozycja wskazuje bieżącą wersję profilu potwierdzoną przez klienta');
END;

-- Przedstawienie i reakcja klienta sprawdzają zmianę, więc propozycja powstaje
-- nieprzedstawiona.
CREATE TRIGGER poszukiwanie_propozycje_powstaje_nieprzedstawiona
BEFORE INSERT ON poszukiwanie_propozycje_wpisy
WHEN NEW.przedstawiono IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'propozycja powstaje nieprzedstawiona');
END;

-- Ceny podobnych transakcji to dokument z plikiem.
CREATE TRIGGER poszukiwanie_propozycje_ceny_rcn_z_plikiem
BEFORE INSERT ON poszukiwanie_propozycje_wpisy
WHEN NEW.ceny_rcn_dokument_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_dokumenty_wpisy
    WHERE id = NEW.ceny_rcn_dokument_id
      AND stan IN ('otrzymany', 'sprawdzony')
  )
BEGIN
  SELECT RAISE(ABORT, 'ceny transakcji z rejestru cen to dokument z plikiem: otrzymany albo sprawdzony');
END;

CREATE TRIGGER poszukiwanie_propozycje_zmienione_ceny_rcn_z_plikiem
BEFORE UPDATE OF ceny_rcn_dokument_id ON poszukiwanie_propozycje_wpisy
WHEN NEW.ceny_rcn_dokument_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_dokumenty_wpisy
    WHERE id = NEW.ceny_rcn_dokument_id
      AND stan IN ('otrzymany', 'sprawdzony')
  )
BEGIN
  SELECT RAISE(ABORT, 'ceny transakcji z rejestru cen to dokument z plikiem: otrzymany albo sprawdzony');
END;

-- Przedstawienie propozycji to praca według zawartej umowy pośrednictwa (art. 15 ust. 3
-- i art. 21 ust. 2 upk).
CREATE TRIGGER poszukiwanie_propozycje_przedstawienie_od_pracy
BEFORE UPDATE OF przedstawiono ON poszukiwanie_propozycje_wpisy
WHEN NEW.przedstawiono IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_umowy_wpisy AS umowa
    JOIN sprawa_umowy_praca_od AS praca ON praca.umowa_id = umowa.id
    WHERE umowa.rodzaj = 'posrednictwo'
      AND umowa.stan = 'zawarta'
      AND NEW.przedstawiono >= praca.od
  )
BEGIN
  SELECT RAISE(ABORT, 'propozycję przedstawia się przy zawartej umowie pośrednictwa, nie wcześniej niż praca od');
END;

-- Źródła i oceny propozycji się nie zmienia; przedstawienie, wynik przeglądania księgi, ceny
-- transakcji i rozbieżności wpisuje się raz, a reakcję zmienia tylko klient, który czekał.
CREATE TRIGGER poszukiwanie_propozycje_wpisy_bez_zmian
BEFORE UPDATE ON poszukiwanie_propozycje_wpisy
WHEN NEW.id IS NOT OLD.id
  OR NEW.profil_wersja_id IS NOT OLD.profil_wersja_id
  OR NEW.zrodlo IS NOT OLD.zrodlo
  OR NEW.ogloszenie_sciezka IS NOT OLD.ogloszenie_sciezka
  OR NEW.ogloszenie_skrot IS NOT OLD.ogloszenie_skrot
  OR NEW.portal IS NOT OLD.portal
  OR NEW.numer IS NOT OLD.numer
  OR NEW.katalog_odnosnik IS NOT OLD.katalog_odnosnik
  OR NEW.katalog_wersja_od IS NOT OLD.katalog_wersja_od
  OR NEW.prowadzi IS NOT OLD.prowadzi
  OR NEW.niepubliczna IS NOT OLD.niepubliczna
  OR NEW.zgodnosc IS NOT OLD.zgodnosc
  OR NEW.niespelnione_zyczenia IS NOT OLD.niespelnione_zyczenia
  OR NEW.cena_ogloszenia IS NOT OLD.cena_ogloszenia
  OR NEW.utworzono IS NOT OLD.utworzono
  OR (OLD.rozbieznosci IS NOT NULL AND NEW.rozbieznosci IS NOT OLD.rozbieznosci)
  OR (OLD.numer_kw IS NOT NULL AND NEW.numer_kw IS NOT OLD.numer_kw)
  OR (OLD.kw_przegladano IS NOT NULL AND (
    NEW.kw_przegladano IS NOT OLD.kw_przegladano
    OR NEW.kw_ogloszajacy_wpisany IS NOT OLD.kw_ogloszajacy_wpisany
    OR NEW.kw_wpisy_iii_iv IS NOT OLD.kw_wpisy_iii_iv
  ))
  OR (OLD.ceny_rcn_dokument_id IS NOT NULL AND NEW.ceny_rcn_dokument_id IS NOT OLD.ceny_rcn_dokument_id)
  OR (OLD.przedstawiono IS NOT NULL AND NEW.przedstawiono IS NOT OLD.przedstawiono)
  OR (OLD.reakcja IN ('chce_obejrzec', 'odrzuca') AND (
    NEW.reakcja IS NOT OLD.reakcja
    OR NEW.reakcja_powod IS NOT OLD.reakcja_powod
  ))
BEGIN
  SELECT RAISE(ABORT, 'propozycji się nie zmienia; przedstawienie, księgę, ceny i rozbieżności wpisuje się raz, a reakcja zmienia się tylko po czekaniu');
END;

-- Prezentacja dotyczy propozycji, którą klient dostał.
CREATE TRIGGER poszukiwanie_propozycje_prezentacja_przedstawionej
BEFORE INSERT ON poszukiwanie_propozycje_prezentacje
WHEN NOT EXISTS (
  SELECT 1
  FROM poszukiwanie_propozycje_wpisy
  WHERE id = NEW.propozycja_id
    AND przedstawiono IS NOT NULL
)
BEGIN
  SELECT RAISE(ABORT, 'prezentacja dotyczy propozycji przedstawionej klientowi');
END;

-- Szerszy zakres danych klienta dla drugiej strony stoi na zatwierdzonej decyzji „inna”, która
-- wskazuje prezentację.
CREATE TRIGGER poszukiwanie_propozycje_szerszy_zakres_za_zgoda
BEFORE INSERT ON poszukiwanie_propozycje_prezentacje
WHEN NEW.szerszy_zakres = 1
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_decyzje_zatwierdzone AS decyzja
    WHERE decyzja.id = NEW.szerszy_zakres_decyzja_id
      AND decyzja.rodzaj = 'inna'
      AND decyzja.dotyczy_tabela = 'poszukiwanie_propozycje_prezentacje'
      AND decyzja.dotyczy_id = NEW.id
  )
BEGIN
  SELECT RAISE(ABORT, 'szerszy zakres danych klienta wymaga zatwierdzonej decyzji inna, która wskazuje prezentację');
END;

-- Prezentacja powstaje umówiona; opinię, notatkę albo odwołanie wpisuje się potem raz. Inny
-- termin to odwołanie tej prezentacji i nowa prezentacja.
CREATE TRIGGER poszukiwanie_propozycje_prezentacja_powstaje_umowiona
BEFORE INSERT ON poszukiwanie_propozycje_prezentacje
WHEN NEW.opinia_klienta IS NOT NULL
  OR NEW.notatka_posrednika IS NOT NULL
  OR NEW.odwolano IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'prezentacja powstaje umówiona, bez opinii, notatki i odwołania');
END;

CREATE TRIGGER poszukiwanie_propozycje_prezentacje_bez_zmian
BEFORE UPDATE ON poszukiwanie_propozycje_prezentacje
WHEN OLD.odwolano IS NOT NULL
  OR NEW.id IS NOT OLD.id
  OR NEW.propozycja_id IS NOT OLD.propozycja_id
  OR NEW.termin IS NOT OLD.termin
  OR NEW.kto_pokazuje IS NOT OLD.kto_pokazuje
  OR NEW.dane_klienta_podane IS NOT OLD.dane_klienta_podane
  OR NEW.szerszy_zakres IS NOT OLD.szerszy_zakres
  OR NEW.szerszy_zakres_decyzja_id IS NOT OLD.szerszy_zakres_decyzja_id
  OR NEW.uczestnicy IS NOT OLD.uczestnicy
  OR (OLD.opinia_klienta IS NOT NULL AND NEW.opinia_klienta IS NOT OLD.opinia_klienta)
  OR (OLD.notatka_posrednika IS NOT NULL AND NEW.notatka_posrednika IS NOT OLD.notatka_posrednika)
BEGIN
  SELECT RAISE(ABORT, 'prezentacji się nie zmienia; opinię, notatkę albo odwołanie wpisuje się raz');
END;

-- Pytanie powstaje bez odpowiedzi; odpowiedź wpisuje się raz, gdy przyjdzie.
CREATE TRIGGER poszukiwanie_propozycje_pytanie_powstaje_otwarte
BEFORE INSERT ON poszukiwanie_propozycje_pytania
WHEN NEW.odpowiedz IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'pytanie powstaje otwarte; odpowiedź wpisuje się, gdy przyjdzie');
END;

CREATE TRIGGER poszukiwanie_propozycje_pytania_bez_zmian
BEFORE UPDATE ON poszukiwanie_propozycje_pytania
WHEN NEW.id IS NOT OLD.id
  OR NEW.prezentacja_id IS NOT OLD.prezentacja_id
  OR NEW.pytanie IS NOT OLD.pytanie
  OR (OLD.odpowiedz IS NOT NULL AND (
    NEW.odpowiedz IS NOT OLD.odpowiedz
    OR NEW.odpowiedz_wplyw_id IS NOT OLD.odpowiedz_wplyw_id
  ))
BEGIN
  SELECT RAISE(ABORT, 'pytania się nie zmienia; odpowiedź wpisuje się raz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER poszukiwanie_propozycje_wpisy_bez_zastepowania
BEFORE INSERT ON poszukiwanie_propozycje_wpisy
WHEN EXISTS (
  SELECT 1
  FROM poszukiwanie_propozycje_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER poszukiwanie_propozycje_wpisy_bez_usuwania
BEFORE DELETE ON poszukiwanie_propozycje_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER poszukiwanie_propozycje_prezentacje_bez_zastepowania
BEFORE INSERT ON poszukiwanie_propozycje_prezentacje
WHEN EXISTS (
  SELECT 1
  FROM poszukiwanie_propozycje_prezentacje
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER poszukiwanie_propozycje_prezentacje_bez_usuwania
BEFORE DELETE ON poszukiwanie_propozycje_prezentacje
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER poszukiwanie_propozycje_pytania_bez_zastepowania
BEFORE INSERT ON poszukiwanie_propozycje_pytania
WHEN EXISTS (
  SELECT 1
  FROM poszukiwanie_propozycje_pytania
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER poszukiwanie_propozycje_pytania_bez_usuwania
BEFORE DELETE ON poszukiwanie_propozycje_pytania
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;
