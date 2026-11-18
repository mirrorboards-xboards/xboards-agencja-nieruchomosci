//! The kind `lokal-krotkoterminowy-kalendarz` of the room `cases/lokal_krotkoterminowy`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind's name.
pub const KIND: &str = "lokal-krotkoterminowy-kalendarz";

/// The migrations of the kind, in the order of their numbers.
pub const MIGRATIONS: &[&str] = &[
    r#"-- Jeden kalendarz lokalu dla wszystkich kanałów, ceny nocy z wyliczeniami i oferty w kanałach
-- sprzedaży z rachunkiem, na który kanał wypłaca przychód. Prowadzi je Zarządca obiektu; blokady
-- na prace wpisuje Koordynator operacji. Rodzaj wskazuje pobyty i zlecenia, więc jest po nich.
--
-- Noc ma jeden wiersz, więc tej samej nocy nie da się sprzedać dwa razy: zajętą noc najpierw się
-- zwalnia. Gdy dwa kanały sprzedały tę samą noc, druga rezerwacja dostaje stan kolizja, a który
-- pobyt zostaje, decyduje człowiek. Wysłanie rezerwacji i blokad do kanałów w ciągu godziny
-- pilnuje program.
CREATE TABLE lokal_krotkoterminowy_kalendarz_noce (
  data TEXT NOT NULL PRIMARY KEY CHECK (data = date(data)),
  stan TEXT NOT NULL CHECK (stan IN ('wolna', 'zarezerwowana', 'wlasciciel', 'prace', 'po_koncu')),
  pobyt_id TEXT REFERENCES lokal_krotkoterminowy_pobyty_wpisy (id),
  zlecenie_id TEXT REFERENCES lokal_krotkoterminowy_zlecenia_wpisy (id),
  -- Zgoda noce_wlasciciela, którą właściciel zgłosił noce dla siebie.
  zgoda_id TEXT REFERENCES sprawa_zgody_wpisy (id),
  zmieniono TEXT NOT NULL,

  CONSTRAINT stan_wskazuje_pobyt_zlecenie_albo_zgode CHECK (
    (stan = 'zarezerwowana') = (pobyt_id IS NOT NULL)
    AND (stan = 'prace') = (zlecenie_id IS NOT NULL)
    AND (stan = 'wlasciciel') = (zgoda_id IS NOT NULL)
  )
) STRICT;

-- Wyliczenia cen: dane wejściowe, które podał Zarządca (sezon, wydarzenia, obłożenie, ceny
-- podobnych lokali), skąd je wziął i z jakiego dnia są, oraz plik wyniku programu.
CREATE TABLE lokal_krotkoterminowy_kalendarz_wyliczenia (
  id TEXT PRIMARY KEY,
  chwila TEXT NOT NULL,
  dane_wejsciowe TEXT NOT NULL CHECK (dane_wejsciowe <> ''),
  zrodla TEXT NOT NULL CHECK (zrodla <> ''),
  sciezka TEXT NOT NULL,
  skrot TEXT NOT NULL
) STRICT;

-- Ceny nocy w kanałach w groszach. Nowa cena to nowy wiersz; bieżąca jest ostatnia ustawiona dla
-- nocy i kanału.
CREATE TABLE lokal_krotkoterminowy_kalendarz_ceny (
  id TEXT PRIMARY KEY,
  data TEXT NOT NULL CHECK (data = date(data)),
  kanal TEXT NOT NULL CHECK (kanal <> '' AND kanal NOT GLOB '*[^a-z_]*'),
  cena INTEGER NOT NULL CHECK (cena > 0),
  ustawiono TEXT NOT NULL,
  wyliczenie_id TEXT REFERENCES lokal_krotkoterminowy_kalendarz_wyliczenia (id),
  -- Zgoda człowieka na cenę poza widełkami.
  decyzja_id TEXT REFERENCES sprawa_decyzje_wpisy (id)
) STRICT;

-- Oferty lokalu w kanałach: konto, numer oferty, dzień publikacji, stan i rachunek do wypłat
-- w kanale. Nowy kanał spoza umowy ma zgodę właściciela zgoda_na_kanal i zgodę człowieka.
CREATE TABLE lokal_krotkoterminowy_kalendarz_kanaly (
  id TEXT PRIMARY KEY,
  kanal TEXT NOT NULL CHECK (kanal <> '' AND kanal NOT GLOB '*[^a-z_]*'),
  konto TEXT NOT NULL CHECK (konto IN ('biura', 'wlasciciela')),
  nazwa_konta TEXT NOT NULL CHECK (nazwa_konta <> ''),
  numer_oferty TEXT,
  opublikowano TEXT,
  stan TEXT NOT NULL CHECK (stan IN ('projekt', 'aktywna', 'zawieszona', 'zamknieta')),
  rachunek_id TEXT REFERENCES sprawa_rachunki_wpisy (id),
  zgoda_id TEXT REFERENCES sprawa_zgody_wpisy (id),
  nowy_kanal_decyzja_id TEXT REFERENCES sprawa_decyzje_wpisy (id),

  CONSTRAINT opublikowana_ma_numer_i_chwile CHECK (
    (stan NOT IN ('aktywna', 'zawieszona') OR opublikowano IS NOT NULL)
    AND (opublikowano IS NULL OR numer_oferty IS NOT NULL)
  )
) STRICT;

CREATE VIEW lokal_krotkoterminowy_kalendarz_ceny_biezace AS
SELECT cena.*
FROM lokal_krotkoterminowy_kalendarz_ceny AS cena
WHERE NOT EXISTS (
  SELECT 1
  FROM lokal_krotkoterminowy_kalendarz_ceny AS nowsza
  WHERE nowsza.data = cena.data
    AND nowsza.kanal = cena.kanal
    AND (nowsza.ustawiono > cena.ustawiono OR (nowsza.ustawiono = cena.ustawiono AND nowsza.rowid > cena.rowid))
);

-- Noc zarezerwowana należy do pobytu, który trwa, czeka albo się zakończył, i leży między dniem
-- przyjazdu a dniem wyjazdu (dzień wyjazdu nie jest nocą pobytu).
CREATE TRIGGER lokal_krotkoterminowy_kalendarz_noc_w_dniach_pobytu
BEFORE INSERT ON lokal_krotkoterminowy_kalendarz_noce
WHEN NEW.stan = 'zarezerwowana'
  AND NOT EXISTS (
    SELECT 1
    FROM lokal_krotkoterminowy_pobyty_wpisy
    WHERE id = NEW.pobyt_id
      AND stan IN ('potwierdzona', 'trwa', 'zakonczona')
      AND NEW.data >= przyjazd
      AND NEW.data < wyjazd
  )
BEGIN
  SELECT RAISE(ABORT, 'noc zarezerwowana należy do potwierdzonego, trwającego albo zakończonego pobytu i leży w jego dniach');
END;

CREATE TRIGGER lokal_krotkoterminowy_kalendarz_zmieniona_noc_w_dniach_pobytu
BEFORE UPDATE ON lokal_krotkoterminowy_kalendarz_noce
WHEN NEW.stan = 'zarezerwowana'
  AND NOT EXISTS (
    SELECT 1
    FROM lokal_krotkoterminowy_pobyty_wpisy
    WHERE id = NEW.pobyt_id
      AND stan IN ('potwierdzona', 'trwa', 'zakonczona')
      AND NEW.data >= przyjazd
      AND NEW.data < wyjazd
  )
BEGIN
  SELECT RAISE(ABORT, 'noc zarezerwowana należy do potwierdzonego, trwającego albo zakończonego pobytu i leży w jego dniach');
END;

-- Noc na prace leży w oknie zlecenia, które nie jest anulowane.
CREATE TRIGGER lokal_krotkoterminowy_kalendarz_noc_w_oknie_zlecenia
BEFORE INSERT ON lokal_krotkoterminowy_kalendarz_noce
WHEN NEW.stan = 'prace'
  AND NOT EXISTS (
    SELECT 1
    FROM lokal_krotkoterminowy_zlecenia_wpisy
    WHERE id = NEW.zlecenie_id
      AND stan IN ('zlecone', 'wstrzymane', 'potwierdzone')
      AND NEW.data >= substr(okno_od, 1, 10)
      AND NEW.data <= substr(okno_do, 1, 10)
  )
BEGIN
  SELECT RAISE(ABORT, 'noc na prace leży w oknie zlecenia, które nie jest anulowane');
END;

CREATE TRIGGER lokal_krotkoterminowy_kalendarz_zmieniona_noc_w_oknie_zlecenia
BEFORE UPDATE ON lokal_krotkoterminowy_kalendarz_noce
WHEN NEW.stan = 'prace'
  AND NOT EXISTS (
    SELECT 1
    FROM lokal_krotkoterminowy_zlecenia_wpisy
    WHERE id = NEW.zlecenie_id
      AND stan IN ('zlecone', 'wstrzymane', 'potwierdzone')
      AND NEW.data >= substr(okno_od, 1, 10)
      AND NEW.data <= substr(okno_do, 1, 10)
  )
BEGIN
  SELECT RAISE(ABORT, 'noc na prace leży w oknie zlecenia, które nie jest anulowane');
END;

-- Noc właściciela wskazuje jego zgłoszenie noce_wlasciciela, złożone z wyprzedzeniem z bieżących
-- warunków umowy; po tym terminie noc zostaje do sprzedaży.
CREATE TRIGGER lokal_krotkoterminowy_kalendarz_noc_wlasciciela_zgloszona_na_czas
BEFORE INSERT ON lokal_krotkoterminowy_kalendarz_noce
WHEN NEW.stan = 'wlasciciel'
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_zgody_wpisy AS zgoda
    JOIN lokal_krotkoterminowy_umowa_warunki_biezace AS warunki
    WHERE zgoda.id = NEW.zgoda_id
      AND zgoda.rodzaj = 'noce_wlasciciela'
      AND substr(zgoda.zlozono, 1, 10) <= date(NEW.data, '-' || warunki.zgloszenie_nocy_dni || ' days')
  )
BEGIN
  SELECT RAISE(ABORT, 'noc właściciela wskazuje zgodę noce_wlasciciela złożoną z wyprzedzeniem z bieżących warunków umowy');
END;

CREATE TRIGGER lokal_krotkoterminowy_kalendarz_zmieniona_noc_wlasciciela_zgloszona_na_czas
BEFORE UPDATE ON lokal_krotkoterminowy_kalendarz_noce
WHEN NEW.stan = 'wlasciciel'
  AND NEW.zgoda_id IS NOT OLD.zgoda_id
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_zgody_wpisy AS zgoda
    JOIN lokal_krotkoterminowy_umowa_warunki_biezace AS warunki
    WHERE zgoda.id = NEW.zgoda_id
      AND zgoda.rodzaj = 'noce_wlasciciela'
      AND substr(zgoda.zlozono, 1, 10) <= date(NEW.data, '-' || warunki.zgloszenie_nocy_dni || ' days')
  )
BEGIN
  SELECT RAISE(ABORT, 'noc właściciela wskazuje zgodę noce_wlasciciela złożoną z wyprzedzeniem z bieżących warunków umowy');
END;

-- Nocy po dacie końca umowy nie można sprzedać: nie jest wolna. Stan po_koncu ma tylko taka noc.
-- Pobyt na te noce wymaga zgody człowieka (lokal-krotkoterminowy-pobyty).
CREATE TRIGGER lokal_krotkoterminowy_kalendarz_noc_po_koncu_umowy
BEFORE INSERT ON lokal_krotkoterminowy_kalendarz_noce
WHEN (
    NEW.stan = 'wolna'
    AND NEW.data > (
      SELECT koniec
      FROM lokal_krotkoterminowy_umowa_lokalu
    )
  )
  OR (
    NEW.stan = 'po_koncu'
    AND NOT coalesce(
      NEW.data > (
        SELECT koniec
        FROM lokal_krotkoterminowy_umowa_lokalu
      ),
      0
    )
  )
BEGIN
  SELECT RAISE(ABORT, 'noc po dacie końca umowy nie jest wolna, a stan po_koncu ma tylko noc po tej dacie');
END;

CREATE TRIGGER lokal_krotkoterminowy_kalendarz_zmieniona_noc_po_koncu_umowy
BEFORE UPDATE ON lokal_krotkoterminowy_kalendarz_noce
WHEN (
    NEW.stan = 'wolna'
    AND NEW.data > (
      SELECT koniec
      FROM lokal_krotkoterminowy_umowa_lokalu
    )
  )
  OR (
    NEW.stan = 'po_koncu'
    AND NOT coalesce(
      NEW.data > (
        SELECT koniec
        FROM lokal_krotkoterminowy_umowa_lokalu
      ),
      0
    )
  )
BEGIN
  SELECT RAISE(ABORT, 'noc po dacie końca umowy nie jest wolna, a stan po_koncu ma tylko noc po tej dacie');
END;

-- Noc pobytu zwalnia się dopiero, gdy pobyt jest odwołany, w kolizji albo po zmianie terminu już
-- jej nie obejmuje. Noce zakończonego pobytu zostają w kalendarzu.
CREATE TRIGGER lokal_krotkoterminowy_kalendarz_noc_pobytu_zwalnia_sie_po_odwolaniu
BEFORE UPDATE ON lokal_krotkoterminowy_kalendarz_noce
WHEN OLD.stan = 'zarezerwowana'
  AND (NEW.stan IS NOT OLD.stan OR NEW.pobyt_id IS NOT OLD.pobyt_id)
  AND EXISTS (
    SELECT 1
    FROM lokal_krotkoterminowy_pobyty_wpisy
    WHERE id = OLD.pobyt_id
      AND stan IN ('potwierdzona', 'trwa', 'zakonczona')
      AND OLD.data >= przyjazd
      AND OLD.data < wyjazd
  )
BEGIN
  SELECT RAISE(ABORT, 'noc pobytu zwalnia się, gdy pobyt jest odwołany, w kolizji albo już jej nie obejmuje');
END;

-- Zajętej nocy nie przejmuje inny pobyt, zlecenie ani zgłoszenie właściciela: najpierw się ją
-- zwalnia. Tak druga rezerwacja tej samej nocy trafia na odmowę i dostaje stan kolizja. Stoi po
-- regule zwolnienia nocy pobytu: SQLite wywołuje najpierw trigger utworzony później, więc odmowa
-- przejęcia nocy podaje tę regułę.
CREATE TRIGGER lokal_krotkoterminowy_kalendarz_zajeta_noc_najpierw_sie_zwalnia
BEFORE UPDATE ON lokal_krotkoterminowy_kalendarz_noce
WHEN OLD.stan IN ('zarezerwowana', 'wlasciciel', 'prace')
  AND NEW.stan <> 'wolna'
  AND (
    NEW.stan IS NOT OLD.stan
    OR NEW.pobyt_id IS NOT OLD.pobyt_id
    OR NEW.zlecenie_id IS NOT OLD.zlecenie_id
    OR NEW.zgoda_id IS NOT OLD.zgoda_id
  )
BEGIN
  SELECT RAISE(ABORT, 'zajęta noc najpierw się zwalnia; nie przejmuje jej inny pobyt, zlecenie ani właściciel');
END;

CREATE TRIGGER lokal_krotkoterminowy_kalendarz_data_nocy_niezmienna
BEFORE UPDATE OF data ON lokal_krotkoterminowy_kalendarz_noce
WHEN NEW.data IS NOT OLD.data
BEGIN
  SELECT RAISE(ABORT, 'data nocy się nie zmienia');
END;

-- Rezerwacja przychodzi z kanału, w którym lokal ma opublikowaną ofertę, i nie sprzed tej
-- publikacji. Rezerwacji ani płatności poza kanałami nie przyjmuje się.
CREATE TRIGGER lokal_krotkoterminowy_kalendarz_rezerwacja_z_kanalu_z_oferta
BEFORE INSERT ON lokal_krotkoterminowy_pobyty_wpisy
WHEN NOT EXISTS (
  SELECT 1
  FROM lokal_krotkoterminowy_kalendarz_kanaly
  WHERE kanal = NEW.kanal
    AND opublikowano IS NOT NULL
    AND opublikowano <= NEW.przyjeto
)
BEGIN
  SELECT RAISE(ABORT, 'rezerwacja przychodzi z kanału, w którym lokal ma opublikowaną ofertę, nie wcześniej niż publikacja');
END;

-- Program nie ustawia ceny poza widełkami z bieżących warunków umowy; taką cenę ustawia Zarządca
-- tylko za zgodą człowieka. Noc bez widełek w swoim okresie traktuje się jak noc poza widełkami.
CREATE TRIGGER lokal_krotkoterminowy_kalendarz_cena_poza_widelkami_za_zgoda
BEFORE INSERT ON lokal_krotkoterminowy_kalendarz_ceny
WHEN NOT coalesce(
    (
      SELECT NEW.cena BETWEEN widelki.cena_min AND widelki.cena_max
      FROM lokal_krotkoterminowy_umowa_widelki AS widelki
      JOIN lokal_krotkoterminowy_umowa_warunki_biezace AS warunki ON warunki.id = widelki.warunki_id
      WHERE NEW.data BETWEEN widelki.od_dnia AND widelki.do_dnia
    ),
    0
  )
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_decyzje_zatwierdzone
    WHERE id = NEW.decyzja_id
      AND rodzaj = 'inna'
      AND dotyczy_tabela = 'lokal_krotkoterminowy_kalendarz_ceny'
      AND dotyczy_id = NEW.id
  )
BEGIN
  SELECT RAISE(ABORT, 'cena poza widełkami z bieżących warunków umowy wskazuje zatwierdzoną decyzję inna, która ją wskazuje');
END;

CREATE TRIGGER lokal_krotkoterminowy_kalendarz_cena_w_kanale_lokalu
BEFORE INSERT ON lokal_krotkoterminowy_kalendarz_ceny
WHEN NOT EXISTS (
    SELECT 1
    FROM lokal_krotkoterminowy_kalendarz_kanaly
    WHERE kanal = NEW.kanal
  )
  AND NOT EXISTS (
    SELECT 1
    FROM lokal_krotkoterminowy_umowa_kanaly AS kanal
    JOIN lokal_krotkoterminowy_umowa_warunki_biezace AS warunki ON warunki.id = kanal.warunki_id
    WHERE kanal.kanal = NEW.kanal
  )
BEGIN
  SELECT RAISE(ABORT, 'cenę ustawia się w kanale z umowy albo w kanale, w którym lokal ma ofertę');
END;

CREATE TRIGGER lokal_krotkoterminowy_kalendarz_ceny_bez_zmian
BEFORE UPDATE ON lokal_krotkoterminowy_kalendarz_ceny
BEGIN
  SELECT RAISE(ABORT, 'ceny się nie zmienia; nowa cena to nowy wiersz');
END;

CREATE TRIGGER lokal_krotkoterminowy_kalendarz_wyliczenia_bez_zmian
BEFORE UPDATE ON lokal_krotkoterminowy_kalendarz_wyliczenia
BEGIN
  SELECT RAISE(ABORT, 'wyliczenia się nie zmienia; nowe wyliczenie to nowy wiersz');
END;

-- Publikacja sprawdza zmianę stanu, więc oferta powstaje jako projekt.
CREATE TRIGGER lokal_krotkoterminowy_kalendarz_oferta_powstaje_jako_projekt
BEFORE INSERT ON lokal_krotkoterminowy_kalendarz_kanaly
WHEN NEW.stan <> 'projekt'
  OR NEW.opublikowano IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'oferta w kanale powstaje jako projekt, bez publikacji');
END;

CREATE TRIGGER lokal_krotkoterminowy_kalendarz_oferta_przejscia_stanu
BEFORE UPDATE OF stan ON lokal_krotkoterminowy_kalendarz_kanaly
WHEN NEW.stan IS NOT OLD.stan
  AND NOT (
    (OLD.stan = 'projekt' AND NEW.stan IN ('aktywna', 'zamknieta'))
    OR (OLD.stan = 'aktywna' AND NEW.stan IN ('zawieszona', 'zamknieta'))
    OR (OLD.stan = 'zawieszona' AND NEW.stan IN ('aktywna', 'zamknieta'))
  )
BEGIN
  SELECT RAISE(ABORT, 'oferta w kanale przechodzi: projekt, aktywna i zawieszona na zmianę, zamknięta');
END;

-- Publikacja to praca według umowy: przy zawartej umowie i nie przed praca_od (R8).
CREATE TRIGGER lokal_krotkoterminowy_kalendarz_publikacja_od_pracy_od
BEFORE UPDATE OF stan ON lokal_krotkoterminowy_kalendarz_kanaly
WHEN NEW.stan = 'aktywna'
  AND OLD.stan = 'projekt'
  AND NOT coalesce(
    (
      SELECT
        umowa.stan = 'zawarta'
        AND NEW.opublikowano >= umowa.praca_od
      FROM lokal_krotkoterminowy_umowa_lokalu AS umowa
    ),
    0
  )
BEGIN
  SELECT RAISE(ABORT, 'ofertę publikuje się przy zawartej umowie o zarządzanie, nie wcześniej niż praca_od');
END;

-- Kanał oferty jest w bieżących warunkach umowy z tym samym kontem albo jest nowym kanałem ze
-- zgodą zgoda_na_kanal właściciela i zgodą człowieka.
CREATE TRIGGER lokal_krotkoterminowy_kalendarz_publikacja_w_kanale_z_umowy_albo_za_zgoda
BEFORE UPDATE OF stan ON lokal_krotkoterminowy_kalendarz_kanaly
WHEN NEW.stan = 'aktywna'
  AND OLD.stan = 'projekt'
  AND NOT EXISTS (
    SELECT 1
    FROM lokal_krotkoterminowy_umowa_kanaly AS kanal
    JOIN lokal_krotkoterminowy_umowa_warunki_biezace AS warunki ON warunki.id = kanal.warunki_id
    WHERE kanal.kanal = NEW.kanal
      AND kanal.konto = NEW.konto
  )
  AND NOT (
    EXISTS (
      SELECT 1
      FROM sprawa_zgody_wpisy AS zgoda
      JOIN sprawa_uczestnicy_role AS rola ON rola.uczestnik_id = zgoda.uczestnik_id
      WHERE zgoda.id = NEW.zgoda_id
        AND zgoda.rodzaj = 'zgoda_na_kanal'
        AND rola.rola IN ('wlasciciel_lokalu', 'wspolwlasciciel')
        AND rola.zakonczono IS NULL
    )
    AND EXISTS (
      SELECT 1
      FROM sprawa_decyzje_zatwierdzone
      WHERE id = NEW.nowy_kanal_decyzja_id
        AND rodzaj = 'inna'
        AND dotyczy_tabela = 'lokal_krotkoterminowy_kalendarz_kanaly'
        AND dotyczy_id = NEW.id
    )
  )
BEGIN
  SELECT RAISE(ABORT, 'ofertę publikuje się w kanale z bieżących warunków umowy z tym samym kontem albo za zgodą zgoda_na_kanal właściciela i zatwierdzoną decyzją inna, która ją wskazuje');
END;

-- Kanał wypłaca przychód tylko na rachunek dopuszczony decyzją po sprawdzeniu Kontrolera (R4).
CREATE TRIGGER lokal_krotkoterminowy_kalendarz_rachunek_kanalu_dopuszczony
BEFORE INSERT ON lokal_krotkoterminowy_kalendarz_kanaly
WHEN NEW.rachunek_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_rachunki_dopuszczone
    WHERE id = NEW.rachunek_id
      AND cel = 'wyplata_w_kanale'
  )
BEGIN
  SELECT RAISE(ABORT, 'rachunek do wypłat w kanale jest dopuszczony decyzją, niezastąpiony i ma cel wyplata_w_kanale');
END;

CREATE TRIGGER lokal_krotkoterminowy_kalendarz_zmieniony_rachunek_kanalu_dopuszczony
BEFORE UPDATE OF rachunek_id ON lokal_krotkoterminowy_kalendarz_kanaly
WHEN NEW.rachunek_id IS NOT NULL
  AND NEW.rachunek_id IS NOT OLD.rachunek_id
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_rachunki_dopuszczone
    WHERE id = NEW.rachunek_id
      AND cel = 'wyplata_w_kanale'
  )
BEGIN
  SELECT RAISE(ABORT, 'rachunek do wypłat w kanale jest dopuszczony decyzją, niezastąpiony i ma cel wyplata_w_kanale');
END;

-- Kanał, konto i publikację wpisuje się raz; zamkniętej oferty się nie zmienia. Nowa oferta to nowy
-- wiersz.
CREATE TRIGGER lokal_krotkoterminowy_kalendarz_oferta_niezmienna
BEFORE UPDATE ON lokal_krotkoterminowy_kalendarz_kanaly
WHEN OLD.stan = 'zamknieta'
  OR NEW.id IS NOT OLD.id
  OR NEW.kanal IS NOT OLD.kanal
  OR NEW.konto IS NOT OLD.konto
  OR NEW.nazwa_konta IS NOT OLD.nazwa_konta
  OR (OLD.numer_oferty IS NOT NULL AND NEW.numer_oferty IS NOT OLD.numer_oferty)
  OR (OLD.opublikowano IS NOT NULL AND NEW.opublikowano IS NOT OLD.opublikowano)
  OR (OLD.stan <> 'projekt' AND NEW.zgoda_id IS NOT OLD.zgoda_id)
  OR (OLD.stan <> 'projekt' AND NEW.nowy_kanal_decyzja_id IS NOT OLD.nowy_kanal_decyzja_id)
BEGIN
  SELECT RAISE(ABORT, 'kanał, konto i publikację oferty wpisuje się raz, a zamkniętej oferty się nie zmienia');
END;

-- Przy zamknięciu lokalu Zarządca zamyka oferty po ostatnim pobycie, przed ostatnim rozliczeniem.
CREATE TRIGGER lokal_krotkoterminowy_kalendarz_rozliczenie_po_zamknieciu_ofert
BEFORE INSERT ON sprawa_metryka_etapy
WHEN NEW.etap = 'ostatnie_rozliczenie'
  AND EXISTS (
    SELECT 1
    FROM lokal_krotkoterminowy_kalendarz_kanaly
    WHERE stan IN ('aktywna', 'zawieszona')
  )
BEGIN
  SELECT RAISE(ABORT, 'etap ostatnie_rozliczenie przychodzi po zamknięciu ofert we wszystkich kanałach');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER lokal_krotkoterminowy_kalendarz_noce_bez_zastepowania
BEFORE INSERT ON lokal_krotkoterminowy_kalendarz_noce
WHEN EXISTS (
  SELECT 1
  FROM lokal_krotkoterminowy_kalendarz_noce
  WHERE data = NEW.data
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER lokal_krotkoterminowy_kalendarz_noce_bez_usuwania
BEFORE DELETE ON lokal_krotkoterminowy_kalendarz_noce
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER lokal_krotkoterminowy_kalendarz_wyliczenia_bez_zastepowania
BEFORE INSERT ON lokal_krotkoterminowy_kalendarz_wyliczenia
WHEN EXISTS (
  SELECT 1
  FROM lokal_krotkoterminowy_kalendarz_wyliczenia
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER lokal_krotkoterminowy_kalendarz_wyliczenia_bez_usuwania
BEFORE DELETE ON lokal_krotkoterminowy_kalendarz_wyliczenia
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER lokal_krotkoterminowy_kalendarz_ceny_bez_zastepowania
BEFORE INSERT ON lokal_krotkoterminowy_kalendarz_ceny
WHEN EXISTS (
  SELECT 1
  FROM lokal_krotkoterminowy_kalendarz_ceny
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER lokal_krotkoterminowy_kalendarz_ceny_bez_usuwania
BEFORE DELETE ON lokal_krotkoterminowy_kalendarz_ceny
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER lokal_krotkoterminowy_kalendarz_kanaly_bez_zastepowania
BEFORE INSERT ON lokal_krotkoterminowy_kalendarz_kanaly
WHEN EXISTS (
  SELECT 1
  FROM lokal_krotkoterminowy_kalendarz_kanaly
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER lokal_krotkoterminowy_kalendarz_kanaly_bez_usuwania
BEFORE DELETE ON lokal_krotkoterminowy_kalendarz_kanaly
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;
"#,
];

/// The values of column `kanaly.konto`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KanalyKonto {
    Biura,
    Wlasciciela,
}

impl KanalyKonto {
    /// Every value, in the order of the catalog.
    pub const ALL: &[KanalyKonto] = &[KanalyKonto::Biura, KanalyKonto::Wlasciciela];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            KanalyKonto::Biura => "biura",
            KanalyKonto::Wlasciciela => "wlasciciela",
        }
    }
}

impl xmeta_room::FromSqlValue for KanalyKonto {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "biura" => Ok(KanalyKonto::Biura),
                    "wlasciciela" => Ok(KanalyKonto::Wlasciciela),
                    other => Err(format!("{other} is no value of KanalyKonto")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for KanalyKonto {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `kanaly.stan`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KanalyStan {
    Projekt,
    Aktywna,
    Zawieszona,
    Zamknieta,
}

impl KanalyStan {
    /// Every value, in the order of the catalog.
    pub const ALL: &[KanalyStan] = &[
        KanalyStan::Projekt,
        KanalyStan::Aktywna,
        KanalyStan::Zawieszona,
        KanalyStan::Zamknieta,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            KanalyStan::Projekt => "projekt",
            KanalyStan::Aktywna => "aktywna",
            KanalyStan::Zawieszona => "zawieszona",
            KanalyStan::Zamknieta => "zamknieta",
        }
    }
}

impl xmeta_room::FromSqlValue for KanalyStan {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "projekt" => Ok(KanalyStan::Projekt),
                    "aktywna" => Ok(KanalyStan::Aktywna),
                    "zawieszona" => Ok(KanalyStan::Zawieszona),
                    "zamknieta" => Ok(KanalyStan::Zamknieta),
                    other => Err(format!("{other} is no value of KanalyStan")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for KanalyStan {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `noce.stan`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NoceStan {
    Wolna,
    Zarezerwowana,
    Wlasciciel,
    Prace,
    PoKoncu,
}

impl NoceStan {
    /// Every value, in the order of the catalog.
    pub const ALL: &[NoceStan] = &[
        NoceStan::Wolna,
        NoceStan::Zarezerwowana,
        NoceStan::Wlasciciel,
        NoceStan::Prace,
        NoceStan::PoKoncu,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            NoceStan::Wolna => "wolna",
            NoceStan::Zarezerwowana => "zarezerwowana",
            NoceStan::Wlasciciel => "wlasciciel",
            NoceStan::Prace => "prace",
            NoceStan::PoKoncu => "po_koncu",
        }
    }
}

impl xmeta_room::FromSqlValue for NoceStan {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "wolna" => Ok(NoceStan::Wolna),
                    "zarezerwowana" => Ok(NoceStan::Zarezerwowana),
                    "wlasciciel" => Ok(NoceStan::Wlasciciel),
                    "prace" => Ok(NoceStan::Prace),
                    "po_koncu" => Ok(NoceStan::PoKoncu),
                    other => Err(format!("{other} is no value of NoceStan")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for NoceStan {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of field `stan` of `zmien_stan_oferty_w_kanale`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Stan {
    Aktywna,
    Zawieszona,
    Zamknieta,
}

impl Stan {
    /// Every value, in the order of the catalog.
    pub const ALL: &[Stan] = &[Stan::Aktywna, Stan::Zawieszona, Stan::Zamknieta];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            Stan::Aktywna => "aktywna",
            Stan::Zawieszona => "zawieszona",
            Stan::Zamknieta => "zamknieta",
        }
    }
}

impl xmeta_room::FromSqlValue for Stan {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "aktywna" => Ok(Stan::Aktywna),
                    "zawieszona" => Ok(Stan::Zawieszona),
                    "zamknieta" => Ok(Stan::Zamknieta),
                    other => Err(format!("{other} is no value of Stan")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for Stan {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// A row of `ceny_nocy`.
#[derive(Debug, Clone, PartialEq)]
pub struct CenyNocyRow {
    pub id: String,
    pub data: String,
    pub kanal: String,
    pub cena: i64,
    pub ustawiono: String,
    pub wyliczenie_id: Option<String>,
    pub decyzja_id: Option<String>,
}

impl CenyNocyRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(CenyNocyRow {
            id: row.get("id")?,
            data: row.get("data")?,
            kanal: row.get("kanal")?,
            cena: row.get("cena")?,
            ustawiono: row.get("ustawiono")?,
            wyliczenie_id: row.get("wyliczenie_id")?,
            decyzja_id: row.get("decyzja_id")?,
        })
    }
}

/// Bieżące ceny nocy w każdym kanale.
pub fn ceny_nocy(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<CenyNocyRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM lokal_krotkoterminowy_kalendarz_ceny_biezace
ORDER BY data, kanal"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(CenyNocyRow::from_row).collect()
}

/// A row of `historia_cen_nocy`.
#[derive(Debug, Clone, PartialEq)]
pub struct HistoriaCenNocyRow {
    pub id: String,
    pub data: String,
    pub kanal: String,
    pub cena: i64,
    pub ustawiono: String,
    pub wyliczenie_id: Option<String>,
    pub decyzja_id: Option<String>,
}

impl HistoriaCenNocyRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(HistoriaCenNocyRow {
            id: row.get("id")?,
            data: row.get("data")?,
            kanal: row.get("kanal")?,
            cena: row.get("cena")?,
            ustawiono: row.get("ustawiono")?,
            wyliczenie_id: row.get("wyliczenie_id")?,
            decyzja_id: row.get("decyzja_id")?,
        })
    }
}

/// Wszystkie ustawione ceny w kolejności ustawienia.
pub fn historia_cen_nocy(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<HistoriaCenNocyRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM lokal_krotkoterminowy_kalendarz_ceny
ORDER BY ustawiono, rowid"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(HistoriaCenNocyRow::from_row).collect()
}

/// The parameters of `ustaw_cene_nocy`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct UstawCeneNocyParams {
    pub id: String,
    pub data: String,
    pub kanal: String,
    pub cena: i64,
    pub ustawiono: String,
    pub wyliczenie_id: Option<String>,
    pub decyzja_id: Option<String>,
}

/// Cena nocy w kanale z wyliczeniem programu; poza widełkami wskazuje zgodę człowieka.
/// Identyfikator (UUID) nadaje aplikacja, bo wskazuje go decyzja.
pub fn ustaw_cene_nocy(
    room: &impl xmeta_room::MetaRoom,
    params: &UstawCeneNocyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO lokal_krotkoterminowy_kalendarz_ceny (id, data, kanal, cena, ustawiono, wyliczenie_id,
  decyzja_id)
VALUES (:id, :data, :kanal, :cena, :ustawiono, :wyliczenie_id, :decyzja_id)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("data", xmeta_room::ToSqlValue::to_sql_value(&params.data)),
                    ("kanal", xmeta_room::ToSqlValue::to_sql_value(&params.kanal)),
                    ("cena", xmeta_room::ToSqlValue::to_sql_value(&params.cena)),
                    (
                        "ustawiono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.ustawiono),
                    ),
                    (
                        "wyliczenie_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wyliczenie_id),
                    ),
                    (
                        "decyzja_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.decyzja_id),
                    ),
                ]),
            },
        ],
    )
}

/// A row of `oferty_w_kanalach`.
#[derive(Debug, Clone, PartialEq)]
pub struct OfertyWKanalachRow {
    pub id: String,
    pub kanal: String,
    pub konto: KanalyKonto,
    pub nazwa_konta: String,
    pub numer_oferty: Option<String>,
    pub opublikowano: Option<String>,
    pub stan: KanalyStan,
    pub rachunek_id: Option<String>,
    pub zgoda_id: Option<String>,
    pub nowy_kanal_decyzja_id: Option<String>,
}

impl OfertyWKanalachRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(OfertyWKanalachRow {
            id: row.get("id")?,
            kanal: row.get("kanal")?,
            konto: row.get("konto")?,
            nazwa_konta: row.get("nazwa_konta")?,
            numer_oferty: row.get("numer_oferty")?,
            opublikowano: row.get("opublikowano")?,
            stan: row.get("stan")?,
            rachunek_id: row.get("rachunek_id")?,
            zgoda_id: row.get("zgoda_id")?,
            nowy_kanal_decyzja_id: row.get("nowy_kanal_decyzja_id")?,
        })
    }
}

/// Oferty lokalu w kanałach sprzedaży.
pub fn oferty_w_kanalach(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<OfertyWKanalachRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM lokal_krotkoterminowy_kalendarz_kanaly
ORDER BY kanal, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(OfertyWKanalachRow::from_row).collect()
}

/// The parameters of `przygotuj_oferte_w_kanale`.
#[derive(Debug, Clone, PartialEq)]
pub struct PrzygotujOferteWKanaleParams {
    pub id: String,
    pub kanal: String,
    pub konto: KanalyKonto,
    pub nazwa_konta: String,
    pub rachunek_id: Option<String>,
    pub zgoda_id: Option<String>,
}

/// Projekt oferty w kanale na koncie biura albo właściciela. Nowy kanał spoza umowy wskazuje zgodę
/// właściciela. Identyfikator (UUID) nadaje aplikacja, bo wskazuje go decyzja o nowym kanale.
pub fn przygotuj_oferte_w_kanale(
    room: &impl xmeta_room::MetaRoom,
    params: &PrzygotujOferteWKanaleParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO lokal_krotkoterminowy_kalendarz_kanaly (id, kanal, konto, nazwa_konta, rachunek_id,
  zgoda_id, stan)
VALUES (:id, :kanal, :konto, :nazwa_konta, :rachunek_id, :zgoda_id, 'projekt')"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("kanal", xmeta_room::ToSqlValue::to_sql_value(&params.kanal)),
                    ("konto", xmeta_room::ToSqlValue::to_sql_value(&params.konto)),
                    (
                        "nazwa_konta",
                        xmeta_room::ToSqlValue::to_sql_value(&params.nazwa_konta),
                    ),
                    (
                        "rachunek_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.rachunek_id),
                    ),
                    ("zgoda_id", xmeta_room::ToSqlValue::to_sql_value(&params.zgoda_id)),
                ]),
            },
        ],
    )
}

/// The parameters of `opublikuj_oferte_w_kanale`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OpublikujOferteWKanaleParams {
    pub numer_oferty: String,
    pub opublikowano: String,
    pub nowy_kanal_decyzja_id: Option<String>,
    pub id: String,
}

/// Oferta opublikowana z numerem w kanale; nowy kanał wskazuje zgodę człowieka.
pub fn opublikuj_oferte_w_kanale(
    room: &impl xmeta_room::MetaRoom,
    params: &OpublikujOferteWKanaleParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE lokal_krotkoterminowy_kalendarz_kanaly
SET stan = 'aktywna',
  numer_oferty = :numer_oferty,
  opublikowano = :opublikowano,
  nowy_kanal_decyzja_id = :nowy_kanal_decyzja_id
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "numer_oferty",
                        xmeta_room::ToSqlValue::to_sql_value(&params.numer_oferty),
                    ),
                    (
                        "opublikowano",
                        xmeta_room::ToSqlValue::to_sql_value(&params.opublikowano),
                    ),
                    (
                        "nowy_kanal_decyzja_id",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.nowy_kanal_decyzja_id,
                        ),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `zmien_stan_oferty_w_kanale`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZmienStanOfertyWKanaleParams {
    pub stan: Stan,
    pub id: String,
}

/// Zawieszenie przez kanał, wznowienie albo zamknięcie oferty.
pub fn zmien_stan_oferty_w_kanale(
    room: &impl xmeta_room::MetaRoom,
    params: &ZmienStanOfertyWKanaleParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE lokal_krotkoterminowy_kalendarz_kanaly
SET stan = :stan
WHERE id = :id"#,
                parameters: Vec::from([
                    ("stan", xmeta_room::ToSqlValue::to_sql_value(&params.stan)),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `wskaz_rachunek_oferty_w_kanale`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WskazRachunekOfertyWKanaleParams {
    pub rachunek_id: Option<String>,
    pub id: String,
}

/// Rachunek, na który kanał wypłaca przychód; zmienia go człowiek po sprawdzeniu Kontrolera.
pub fn wskaz_rachunek_oferty_w_kanale(
    room: &impl xmeta_room::MetaRoom,
    params: &WskazRachunekOfertyWKanaleParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE lokal_krotkoterminowy_kalendarz_kanaly
SET rachunek_id = :rachunek_id
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "rachunek_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.rachunek_id),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// A row of `noce`.
#[derive(Debug, Clone, PartialEq)]
pub struct NoceRow {
    pub data: String,
    pub stan: NoceStan,
    pub pobyt_id: Option<String>,
    pub zlecenie_id: Option<String>,
    pub zgoda_id: Option<String>,
    pub zmieniono: String,
}

impl NoceRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(NoceRow {
            data: row.get("data")?,
            stan: row.get("stan")?,
            pobyt_id: row.get("pobyt_id")?,
            zlecenie_id: row.get("zlecenie_id")?,
            zgoda_id: row.get("zgoda_id")?,
            zmieniono: row.get("zmieniono")?,
        })
    }
}

/// Kalendarz lokalu: noce w kolejności dat.
pub fn noce(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<NoceRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM lokal_krotkoterminowy_kalendarz_noce
ORDER BY data"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(NoceRow::from_row).collect()
}

/// The parameters of `wpisz_noc`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszNocParams {
    pub data: String,
    pub stan: NoceStan,
    pub pobyt_id: Option<String>,
    pub zlecenie_id: Option<String>,
    pub zgoda_id: Option<String>,
    pub zmieniono: String,
}

/// Pierwszy wpis nocy w kalendarzu: wolna, zarezerwowana dla pobytu, na prace, dla właściciela albo
/// po końcu umowy.
pub fn wpisz_noc(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszNocParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO lokal_krotkoterminowy_kalendarz_noce (data, stan, pobyt_id, zlecenie_id, zgoda_id,
  zmieniono)
VALUES (:data, :stan, :pobyt_id, :zlecenie_id, :zgoda_id, :zmieniono)"#,
                parameters: Vec::from([
                    ("data", xmeta_room::ToSqlValue::to_sql_value(&params.data)),
                    ("stan", xmeta_room::ToSqlValue::to_sql_value(&params.stan)),
                    ("pobyt_id", xmeta_room::ToSqlValue::to_sql_value(&params.pobyt_id)),
                    (
                        "zlecenie_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zlecenie_id),
                    ),
                    ("zgoda_id", xmeta_room::ToSqlValue::to_sql_value(&params.zgoda_id)),
                    (
                        "zmieniono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zmieniono),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `zmien_noc`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZmienNocParams {
    pub stan: NoceStan,
    pub pobyt_id: Option<String>,
    pub zlecenie_id: Option<String>,
    pub zgoda_id: Option<String>,
    pub zmieniono: String,
    pub data: String,
}

/// Nowy stan nocy; zajętą noc najpierw się zwalnia (stan wolna).
pub fn zmien_noc(
    room: &impl xmeta_room::MetaRoom,
    params: &ZmienNocParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE lokal_krotkoterminowy_kalendarz_noce
SET stan = :stan,
  pobyt_id = :pobyt_id,
  zlecenie_id = :zlecenie_id,
  zgoda_id = :zgoda_id,
  zmieniono = :zmieniono
WHERE data = :data"#,
                parameters: Vec::from([
                    ("stan", xmeta_room::ToSqlValue::to_sql_value(&params.stan)),
                    ("pobyt_id", xmeta_room::ToSqlValue::to_sql_value(&params.pobyt_id)),
                    (
                        "zlecenie_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zlecenie_id),
                    ),
                    ("zgoda_id", xmeta_room::ToSqlValue::to_sql_value(&params.zgoda_id)),
                    (
                        "zmieniono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zmieniono),
                    ),
                    ("data", xmeta_room::ToSqlValue::to_sql_value(&params.data)),
                ]),
            },
        ],
    )
}

/// A row of `wyliczenia_cen`.
#[derive(Debug, Clone, PartialEq)]
pub struct WyliczeniaCenRow {
    pub id: String,
    pub chwila: String,
    pub dane_wejsciowe: String,
    pub zrodla: String,
    pub sciezka: String,
    pub skrot: String,
}

impl WyliczeniaCenRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(WyliczeniaCenRow {
            id: row.get("id")?,
            chwila: row.get("chwila")?,
            dane_wejsciowe: row.get("dane_wejsciowe")?,
            zrodla: row.get("zrodla")?,
            sciezka: row.get("sciezka")?,
            skrot: row.get("skrot")?,
        })
    }
}

/// Wyliczenia cen w kolejności czasu.
pub fn wyliczenia_cen(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<WyliczeniaCenRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM lokal_krotkoterminowy_kalendarz_wyliczenia
ORDER BY chwila, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(WyliczeniaCenRow::from_row).collect()
}

/// The parameters of `wpisz_wyliczenie_cen`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpiszWyliczenieCenParams {
    pub id: String,
    pub chwila: String,
    pub dane_wejsciowe: String,
    pub zrodla: String,
    pub sciezka: String,
    pub skrot: String,
}

/// Dane wejściowe wyliczenia, ich źródła z datą i plik wyniku programu.
pub fn wpisz_wyliczenie_cen(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszWyliczenieCenParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO lokal_krotkoterminowy_kalendarz_wyliczenia (id, chwila, dane_wejsciowe, zrodla, sciezka,
  skrot)
VALUES (:id, :chwila, :dane_wejsciowe, :zrodla, :sciezka, :skrot)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("chwila", xmeta_room::ToSqlValue::to_sql_value(&params.chwila)),
                    (
                        "dane_wejsciowe",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dane_wejsciowe),
                    ),
                    ("zrodla", xmeta_room::ToSqlValue::to_sql_value(&params.zrodla)),
                    ("sciezka", xmeta_room::ToSqlValue::to_sql_value(&params.sciezka)),
                    ("skrot", xmeta_room::ToSqlValue::to_sql_value(&params.skrot)),
                ]),
            },
        ],
    )
}
