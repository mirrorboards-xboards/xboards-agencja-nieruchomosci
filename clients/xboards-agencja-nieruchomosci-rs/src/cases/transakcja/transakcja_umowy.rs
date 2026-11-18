//! The kind `transakcja-umowy` of the room `cases/transakcja`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind's name.
pub const KIND: &str = "transakcja-umowy";

/// The migrations of the kind, in the order of their numbers.
pub const MIGRATIONS: &[&str] = &[
    r#"-- Umowy stron transakcji: kancelaria notarialna, umowy rezerwacyjna, przedwstępna, warunkowa
-- i przenosząca własność, ich strony, płatności i prawa pierwokupu. Biuro nie jest stroną tych
-- umów: projekty pisemne przygotowuje według wzorów, projekt aktu przygotowuje kancelaria,
-- a podpisują strony. Pisze Koordynator transakcji.
--
-- Umowa powstaje jako projekt i zostaje zawarta albo nie dochodzi do skutku; zawarta może zostać
-- rozwiązana. Od wyjścia z projektu jej treść (rodzaj, forma, cena, terminy, dokumenty) się nie
-- zmienia; po akcie dochodzi tylko obserwacja księgi wieczystej (wniosek i wpis kupującego).
--
-- Kredyt kupującego to płatność ze źródłem kredyt, z bankiem i warunkami wypłaty. Termin decyzji
-- kredytowej trzyma rdzeń (sprawa-terminy). Dochody kupującego i granice ceny stron do teczki nie
-- trafiają (R1): nie mają tu kolumn.
--
-- Reguła, że umowa zostaje zawarta tylko z aktualną analizą swojego projektu, leży w migracji
-- transakcja-analizy (transakcja_analizy_zawarcie_po_analizie_projektu).

-- Kancelaria, która przygotowuje akt, i pokój wymiany z nią. Pokój otwiera się tylko z kancelarią
-- pracującą w XNotariuszu i tylko za zgodą człowieka; decyzja wskazuje wiersz kancelarii, więc
-- wiersz ma UUID. Zmiana kancelarii kończy bieżący wiersz (do, powód) i zaczyna nowy.
CREATE TABLE transakcja_umowy_kancelarie (
  id TEXT PRIMARY KEY,
  nazwa TEXT NOT NULL CHECK (nazwa <> ''),
  adres TEXT,
  -- Notariusz, który sporządzi akt, np. „notariusz Maria Wiśniewska”.
  notariusz TEXT,
  -- Kancelaria pracuje w XNotariuszu, więc może mieć z biurem pokój wymiany.
  xnotariusz INTEGER NOT NULL CHECK (xnotariusz IN (0, 1)),
  -- Zatwierdzona decyzja inna o otwarciu pokoju wymiany i odnośnik do pokoju, tekstem.
  otwarcie_decyzja_id TEXT REFERENCES sprawa_decyzje_wpisy (id),
  pokoj_odnosnik TEXT,
  od TEXT NOT NULL,
  do TEXT,
  powod_zmiany TEXT,

  CONSTRAINT pokoj_razem_z_decyzja CHECK ((pokoj_odnosnik IS NULL) = (otwarcie_decyzja_id IS NULL)),
  CONSTRAINT pokoj_tylko_z_xnotariuszem CHECK (pokoj_odnosnik IS NULL OR xnotariusz = 1),
  CONSTRAINT koniec_z_powodem CHECK ((do IS NULL) = (powod_zmiany IS NULL)),
  CONSTRAINT koniec_nie_przed_poczatkiem CHECK (do IS NULL OR do >= od)
) STRICT;

-- Umowy stron. Umowa przenosząca przy pierwokupie wskazuje umowę warunkową (oparta_na_id), a umowa
-- przyrzeczona może wskazać przedwstępną.
CREATE TABLE transakcja_umowy_wpisy (
  id TEXT PRIMARY KEY,
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('rezerwacyjna', 'przedwstepna', 'warunkowa', 'przenoszaca')),
  forma TEXT NOT NULL CHECK (forma IN ('pisemna', 'akt_notarialny')),
  stan TEXT NOT NULL CHECK (stan IN ('projekt', 'zawarta', 'nie_doszla', 'rozwiazana')),
  oparta_na_id TEXT REFERENCES transakcja_umowy_wpisy (id),
  koniec_powod TEXT,

  -- Dokumenty i kancelaria: projekt, który ocenia Analityk, podpisany egzemplarz albo wypis aktu,
  -- chwila zawarcia i numer repertorium A.
  kancelaria_id TEXT REFERENCES transakcja_umowy_kancelarie (id),
  projekt_dokument_id TEXT REFERENCES sprawa_dokumenty_wpisy (id),
  dokument_id TEXT REFERENCES sprawa_dokumenty_wpisy (id),
  zawarto TEXT,
  repertorium TEXT,

  -- Warunki: cena w groszach, terminy z umowy (umowa przyrzeczona, zapłata, wydanie) i inne
  -- warunki, np. uzyskanie kredytu albo wykreślenie hipoteki.
  cena INTEGER CHECK (cena > 0),
  termin_przyrzeczonej TEXT,
  termin_zaplaty TEXT,
  termin_wydania TEXT,
  warunki TEXT,

  -- Księga wieczysta po akcie: kiedy notariusz złożył wniosek (art. 92 § 4 PoN) i kiedy sąd
  -- wpisał kupującego.
  wniosek_kw_zlozono TEXT,
  wpis_kw TEXT,

  -- Art. 158 KC: umowa przenosząca własność nieruchomości, także warunkowa, wymaga aktu
  -- notarialnego. Przedwstępną strony zawierają na piśmie albo w akcie (art. 390 § 2 KC),
  -- a rezerwacyjną na piśmie.
  CONSTRAINT forma_wedlug_rodzaju CHECK (
    CASE rodzaj
      WHEN 'warunkowa' THEN forma = 'akt_notarialny'
      WHEN 'przenoszaca' THEN forma = 'akt_notarialny'
      WHEN 'rezerwacyjna' THEN forma = 'pisemna'
      ELSE 1
    END
  ),
  CONSTRAINT koniec_ma_powod CHECK ((stan IN ('nie_doszla', 'rozwiazana')) = (koniec_powod IS NOT NULL)),
  CONSTRAINT zawarta_ma_dokument_i_chwile CHECK (
    (zawarto IS NULL) = (dokument_id IS NULL)
    AND (stan NOT IN ('zawarta', 'rozwiazana') OR zawarto IS NOT NULL)
  ),
  CONSTRAINT akt_ma_kancelarie_i_repertorium CHECK (
    zawarto IS NULL
    OR forma <> 'akt_notarialny'
    OR (kancelaria_id IS NOT NULL AND repertorium IS NOT NULL)
  ),
  CONSTRAINT repertorium_tylko_przy_akcie CHECK (repertorium IS NULL OR forma = 'akt_notarialny'),
  CONSTRAINT ksiega_po_zawartej_umowie_przenoszacej CHECK (
    wniosek_kw_zlozono IS NULL OR (rodzaj = 'przenoszaca' AND zawarto IS NOT NULL)
  ),
  CONSTRAINT wpis_po_wniosku CHECK (wpis_kw IS NULL OR wniosek_kw_zlozono IS NOT NULL),
  CONSTRAINT nie_opiera_sie_na_sobie CHECK (oparta_na_id IS NOT id)
) STRICT;

-- Strony umowy: kto sprzedaje, kto kupuje i który małżonek wyraża zgodę (art. 37 § 1 pkt 1 KRO).
-- Strona, która podpisuje przez pełnomocnika, wskazuje jego umocowanie.
CREATE TABLE transakcja_umowy_strony (
  -- Klucz nadany przez bazę jest dodatni; zob. transakcja_umowy_strony_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  umowa_id TEXT NOT NULL REFERENCES transakcja_umowy_wpisy (id),
  uczestnik_id TEXT NOT NULL REFERENCES sprawa_uczestnicy_wpisy (id),
  strona TEXT NOT NULL CHECK (strona IN ('sprzedajacy', 'kupujacy', 'malzonek')),
  umocowanie_id TEXT REFERENCES sprawa_umocowania_wpisy (id),

  UNIQUE (umowa_id, uczestnik_id)
) STRICT;

-- Płatności z umowy: opłata rezerwacyjna, zadatek, zaliczka, cena i spłata hipoteki sprzedającego.
-- Wpłata trafia na rachunek dopuszczony decyzją (R4); potwierdzenie to dokument z banku.
CREATE TABLE transakcja_umowy_platnosci (
  id TEXT PRIMARY KEY,
  umowa_id TEXT NOT NULL REFERENCES transakcja_umowy_wpisy (id),
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('oplata_rezerwacyjna', 'zadatek', 'zaliczka', 'cena', 'splata_hipoteki')),
  -- Jak umowa nazywa wpłatę i co z tego wynika, np. „zadatek w rozumieniu art. 394 KC”; ocenia
  -- to Analityk.
  charakter TEXT,
  kwota INTEGER NOT NULL CHECK (kwota > 0),
  termin TEXT,
  zrodlo TEXT NOT NULL CHECK (zrodlo IN ('wlasne', 'kredyt')),
  -- Bank kupującego i warunki wypłaty kredytu; tylko przy kredycie.
  bank TEXT,
  warunki_wyplaty TEXT,
  rachunek_id TEXT REFERENCES sprawa_rachunki_wpisy (id),
  wplacono TEXT,
  potwierdzenie_dokument_id TEXT REFERENCES sprawa_dokumenty_wpisy (id),

  CONSTRAINT kredyt_z_bankiem_i_warunkami CHECK (
    (zrodlo = 'kredyt') = (bank IS NOT NULL)
    AND (zrodlo = 'kredyt') = (warunki_wyplaty IS NOT NULL)
  ),
  CONSTRAINT wplata_z_rachunkiem_i_potwierdzeniem CHECK (
    (wplacono IS NULL) = (potwierdzenie_dokument_id IS NULL)
    AND (wplacono IS NULL OR rachunek_id IS NOT NULL)
  )
) STRICT;

-- Prawa pierwokupu, np. gminy, dzierżawcy albo Krajowego Ośrodka Wsparcia Rolnictwa. Wynik
-- przychodzi od kancelarii, która zawiadamia uprawnionego (art. 110 ust. 3 ugn, art. 3 ust. 2 ukur).
CREATE TABLE transakcja_umowy_pierwokupy (
  id TEXT PRIMARY KEY,
  uprawniony TEXT NOT NULL CHECK (uprawniony <> ''),
  -- Przepis, z którego wynika prawo, np. „art. 109 ust. 1 pkt 3 ugn”.
  podstawa TEXT NOT NULL CHECK (podstawa <> ''),
  zawiadomiono TEXT,
  wynik TEXT NOT NULL CHECK (wynik IN ('czeka', 'nie_wykonano', 'wykonano', 'uplynal')),
  -- Wiadomość z kancelarii o wyniku.
  zrodlo_wplyw_id TEXT REFERENCES sprawa_korespondencja_wplywy (id),

  -- Termin na wykonanie biegnie od zawiadomienia (art. 598 § 2 KC), a wynik zna kancelaria.
  CONSTRAINT wynik_po_zawiadomieniu_z_wiadomoscia CHECK (
    wynik = 'czeka' OR (zawiadomiono IS NOT NULL AND zrodlo_wplyw_id IS NOT NULL)
  )
) STRICT;

-- Kancelaria -------------------------------------------------------------------------------------

-- Pokój otwiera się po wpisie, za zgodą człowieka, a nowa kancelaria zaczyna się po końcu
-- poprzedniej; wiersz powstaje więc bez pokoju i bez końca.
CREATE TRIGGER transakcja_umowy_kancelaria_powstaje_biezaca
BEFORE INSERT ON transakcja_umowy_kancelarie
WHEN NEW.otwarcie_decyzja_id IS NOT NULL
  OR NEW.do IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'kancelaria powstaje bez pokoju wymiany i bez końca');
END;

CREATE TRIGGER transakcja_umowy_jedna_biezaca_kancelaria
BEFORE INSERT ON transakcja_umowy_kancelarie
WHEN EXISTS (
  SELECT 1
  FROM transakcja_umowy_kancelarie
  WHERE do IS NULL
)
BEGIN
  SELECT RAISE(ABORT, 'teczka ma jedną bieżącą kancelarię; poprzednią kończy data do z powodem zmiany');
END;

CREATE TRIGGER transakcja_umowy_pokoj_wymiany_za_zgoda
BEFORE UPDATE OF otwarcie_decyzja_id, pokoj_odnosnik ON transakcja_umowy_kancelarie
WHEN NEW.otwarcie_decyzja_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_decyzje_zatwierdzone AS decyzja
    WHERE decyzja.id = NEW.otwarcie_decyzja_id
      AND decyzja.rodzaj = 'inna'
      AND decyzja.dotyczy_tabela = 'transakcja_umowy_kancelarie'
      AND decyzja.dotyczy_id = NEW.id
  )
BEGIN
  SELECT RAISE(ABORT, 'pokój wymiany otwiera tylko zatwierdzona decyzja inna, która wskazuje tę kancelarię');
END;

-- Decyzja o pokoju dotyczy tej kancelarii, a pokój i koniec wpisuje się raz.
CREATE TRIGGER transakcja_umowy_kancelaria_bez_zmian
BEFORE UPDATE ON transakcja_umowy_kancelarie
WHEN OLD.do IS NOT NULL
  OR NEW.id IS NOT OLD.id
  OR (OLD.otwarcie_decyzja_id IS NOT NULL AND (
    NEW.otwarcie_decyzja_id IS NOT OLD.otwarcie_decyzja_id
    OR NEW.pokoj_odnosnik IS NOT OLD.pokoj_odnosnik
    OR NEW.nazwa IS NOT OLD.nazwa
    OR NEW.adres IS NOT OLD.adres
    OR NEW.notariusz IS NOT OLD.notariusz
    OR NEW.xnotariusz IS NOT OLD.xnotariusz
    OR NEW.od IS NOT OLD.od
  ))
BEGIN
  SELECT RAISE(ABORT, 'kancelarii z pokojem wymiany ani zakończonej się nie zmienia; zmiana kancelarii to nowy wiersz');
END;

-- Umowy -------------------------------------------------------------------------------------------

-- Reguły zawarcia sprawdzają zmianę stanu, więc umowa powstaje jako projekt bez danych zawarcia.
CREATE TRIGGER transakcja_umowy_powstaje_jako_projekt
BEFORE INSERT ON transakcja_umowy_wpisy
WHEN NEW.stan <> 'projekt'
  OR NEW.zawarto IS NOT NULL
  OR NEW.repertorium IS NOT NULL
  OR NEW.wniosek_kw_zlozono IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'umowa transakcji powstaje jako projekt, bez danych zawarcia');
END;

CREATE TRIGGER transakcja_umowy_przejscia_stanu
BEFORE UPDATE OF stan ON transakcja_umowy_wpisy
WHEN NEW.stan IS NOT OLD.stan
  AND NOT (
    (OLD.stan = 'projekt' AND NEW.stan IN ('zawarta', 'nie_doszla'))
    OR (OLD.stan = 'zawarta' AND NEW.stan = 'rozwiazana')
  )
BEGIN
  SELECT RAISE(ABORT, 'umowa transakcji przechodzi z projektu do zawartej albo niedoszłej, a zawarta do rozwiązanej');
END;

-- Zawarta umowa ma podpisany egzemplarz albo wypis aktu jako dokument teczki z plikiem.
CREATE TRIGGER transakcja_umowy_zawarta_z_dokumentem
BEFORE UPDATE OF stan ON transakcja_umowy_wpisy
WHEN NEW.stan = 'zawarta'
  AND OLD.stan = 'projekt'
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_dokumenty_wpisy
    WHERE id = NEW.dokument_id
      AND rodzaj = CASE NEW.forma WHEN 'akt_notarialny' THEN 'akt_notarialny' ELSE 'umowa' END
      AND stan IN ('otrzymany', 'sprawdzony')
  )
BEGIN
  SELECT RAISE(ABORT, 'zawarta umowa wskazuje otrzymany dokument: wypis aktu notarialnego albo podpisaną umowę');
END;

-- Art. 597 § 1 i art. 598 § 2 KC: przy pierwokupie strony zawierają najpierw umowę warunkową,
-- a przenoszącą dopiero, gdy żaden uprawniony nie wykonał prawa (nie wykonał albo termin upłynął).
CREATE TRIGGER transakcja_umowy_przenoszaca_po_pierwokupie
BEFORE UPDATE OF stan ON transakcja_umowy_wpisy
WHEN NEW.stan = 'zawarta'
  AND OLD.stan = 'projekt'
  AND NEW.rodzaj = 'przenoszaca'
  AND EXISTS (
    SELECT 1
    FROM transakcja_umowy_pierwokupy
  )
  AND (
    NOT EXISTS (
      SELECT 1
      FROM transakcja_umowy_wpisy
      WHERE id = NEW.oparta_na_id
        AND rodzaj = 'warunkowa'
        AND stan = 'zawarta'
    )
    OR EXISTS (
      SELECT 1
      FROM transakcja_umowy_pierwokupy
      WHERE wynik NOT IN ('nie_wykonano', 'uplynal')
    )
  )
BEGIN
  SELECT RAISE(ABORT, 'przy pierwokupie umowa przenosząca wymaga zawartej umowy warunkowej, a każdy pierwokup nie_wykonano albo uplynal');
END;

-- Art. 599 § 2 KC: gdy uprawniony wykonał pierwokup, sprzedaż na rzecz kupującego jest nieważna,
-- więc projektu umowy przenoszącej już się nie zaczyna.
CREATE TRIGGER transakcja_umowy_bez_przenoszacej_po_wykonanym_pierwokupie
BEFORE INSERT ON transakcja_umowy_wpisy
WHEN NEW.rodzaj = 'przenoszaca'
  AND EXISTS (
    SELECT 1
    FROM transakcja_umowy_pierwokupy
    WHERE wynik = 'wykonano'
  )
BEGIN
  SELECT RAISE(ABORT, 'uprawniony wykonał pierwokup, więc umowy przenoszącej nie ma (art. 599 § 2 KC)');
END;

-- Pierwokup wpisuje się, zanim strony zawrą umowę przenoszącą; później ominąłby jej regułę.
CREATE TRIGGER transakcja_umowy_pierwokup_przed_umowa_przenoszaca
BEFORE INSERT ON transakcja_umowy_pierwokupy
WHEN EXISTS (
  SELECT 1
  FROM transakcja_umowy_wpisy
  WHERE rodzaj = 'przenoszaca'
    AND zawarto IS NOT NULL
)
BEGIN
  SELECT RAISE(ABORT, 'pierwokup wpisuje się przed zawarciem umowy przenoszącej');
END;

-- Po wyjściu z projektu umowa jest tym, co strony podpisały. Zmienia się już tylko stan (zawarta
-- się rozwiązuje) z powodem oraz obserwacja księgi, którą wpisuje się raz.
CREATE TRIGGER transakcja_umowy_tresc_poza_projektem_niezmienna
BEFORE UPDATE ON transakcja_umowy_wpisy
WHEN OLD.stan <> 'projekt'
  AND (
    NEW.id IS NOT OLD.id
    OR NEW.rodzaj IS NOT OLD.rodzaj
    OR NEW.forma IS NOT OLD.forma
    OR NEW.oparta_na_id IS NOT OLD.oparta_na_id
    OR NEW.kancelaria_id IS NOT OLD.kancelaria_id
    OR NEW.projekt_dokument_id IS NOT OLD.projekt_dokument_id
    OR NEW.dokument_id IS NOT OLD.dokument_id
    OR NEW.zawarto IS NOT OLD.zawarto
    OR NEW.repertorium IS NOT OLD.repertorium
    OR NEW.cena IS NOT OLD.cena
    OR NEW.termin_przyrzeczonej IS NOT OLD.termin_przyrzeczonej
    OR NEW.termin_zaplaty IS NOT OLD.termin_zaplaty
    OR NEW.termin_wydania IS NOT OLD.termin_wydania
    OR NEW.warunki IS NOT OLD.warunki
    OR (OLD.koniec_powod IS NOT NULL AND NEW.koniec_powod IS NOT OLD.koniec_powod)
    OR (OLD.wniosek_kw_zlozono IS NOT NULL AND NEW.wniosek_kw_zlozono IS NOT OLD.wniosek_kw_zlozono)
    OR (OLD.wpis_kw IS NOT NULL AND NEW.wpis_kw IS NOT OLD.wpis_kw)
  )
BEGIN
  SELECT RAISE(ABORT, 'treść umowy transakcji zmienia się tylko w projekcie; potem tylko stan i raz wpisany wniosek i wpis do księgi');
END;

-- Faza przekazania przychodzi po zawarciu umowy przenoszącej własność.
CREATE TRIGGER transakcja_umowy_przekazanie_po_umowie_przenoszacej
BEFORE INSERT ON sprawa_metryka_etapy
WHEN NEW.etap = 'przekazanie'
  AND NOT EXISTS (
    SELECT 1
    FROM transakcja_umowy_wpisy
    WHERE rodzaj = 'przenoszaca'
      AND stan = 'zawarta'
  )
BEGIN
  SELECT RAISE(ABORT, 'faza przekazania wymaga zawartej umowy przenoszącej własność');
END;

-- Strony ------------------------------------------------------------------------------------------

-- Strony są częścią treści, którą strony podpisują.
CREATE TRIGGER transakcja_umowy_strony_tylko_w_projekcie
BEFORE INSERT ON transakcja_umowy_strony
WHEN (
  SELECT stan <> 'projekt'
  FROM transakcja_umowy_wpisy
  WHERE id = NEW.umowa_id
)
BEGIN
  SELECT RAISE(ABORT, 'strony dopisuje się tylko do projektu umowy transakcji');
END;

CREATE TRIGGER transakcja_umowy_strony_bez_zmian
BEFORE UPDATE ON transakcja_umowy_strony
BEGIN
  SELECT RAISE(ABORT, 'strony umowy transakcji się nie zmienia; nowa strona to nowy wiersz projektu');
END;

-- Pełnomocnik działa z umocowania od tej strony. Art. 99 § 1 KC: do umowy, która wymaga aktu
-- notarialnego (warunkowa i przenosząca), pełnomocnictwo ma formę aktu; podmiot działa przez
-- reprezentację z rejestru.
CREATE TRIGGER transakcja_umowy_pelnomocnik_strony_z_umocowaniem
BEFORE INSERT ON transakcja_umowy_strony
WHEN NEW.umocowanie_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_umocowania_wpisy AS umocowanie
    JOIN transakcja_umowy_wpisy AS umowa ON umowa.id = NEW.umowa_id
    WHERE umocowanie.id = NEW.umocowanie_id
      AND umocowanie.mocodawca_uczestnik_id = NEW.uczestnik_id
      AND (
        umowa.rodzaj NOT IN ('warunkowa', 'przenoszaca')
        OR umocowanie.rodzaj = 'reprezentacja'
        OR umocowanie.forma = 'akt_notarialny'
      )
  )
BEGIN
  SELECT RAISE(ABORT, 'pełnomocnik strony działa z jej umocowania, a do umowy warunkowej i przenoszącej pełnomocnictwo ma formę aktu notarialnego (art. 99 § 1 KC)');
END;

-- To samo, gdy zmienia się umocowanie, którym strona już podpisuje umowę w formie aktu.
CREATE TRIGGER transakcja_umowy_umocowanie_strony_aktu_zostaje_aktem
BEFORE UPDATE OF rodzaj, forma, mocodawca_uczestnik_id ON sprawa_umocowania_wpisy
WHEN EXISTS (
  SELECT 1
  FROM transakcja_umowy_strony AS strona
  JOIN transakcja_umowy_wpisy AS umowa ON umowa.id = strona.umowa_id
  WHERE strona.umocowanie_id = NEW.id
    AND (
      NEW.mocodawca_uczestnik_id IS NOT strona.uczestnik_id
      OR (
        umowa.rodzaj IN ('warunkowa', 'przenoszaca')
        AND NEW.rodzaj = 'pelnomocnictwo'
        AND NEW.forma IS NOT 'akt_notarialny'
      )
    )
)
BEGIN
  SELECT RAISE(ABORT, 'pełnomocnik strony działa z jej umocowania, a do umowy warunkowej i przenoszącej pełnomocnictwo ma formę aktu notarialnego (art. 99 § 1 KC)');
END;

-- Płatności --------------------------------------------------------------------------------------

-- Płatność jest postanowieniem umowy, więc powstaje w projekcie i jeszcze niezapłacona.
CREATE TRIGGER transakcja_umowy_platnosc_w_projekcie
BEFORE INSERT ON transakcja_umowy_platnosci
WHEN NEW.wplacono IS NOT NULL
  OR (
    SELECT stan <> 'projekt'
    FROM transakcja_umowy_wpisy
    WHERE id = NEW.umowa_id
  )
BEGIN
  SELECT RAISE(ABORT, 'płatność dopisuje się do projektu umowy transakcji, jeszcze niezapłaconą');
END;

-- R4: rachunek, który trafia do umowy, jest dopuszczony decyzją i niezastąpiony; przy wpłacie
-- baza sprawdza to jeszcze raz, bo rachunek mógł zostać zastąpiony.
CREATE TRIGGER transakcja_umowy_platnosc_na_dopuszczony_rachunek
BEFORE INSERT ON transakcja_umowy_platnosci
WHEN NEW.rachunek_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_rachunki_dopuszczone
    WHERE id = NEW.rachunek_id
  )
BEGIN
  SELECT RAISE(ABORT, 'płatność wskazuje rachunek dopuszczony decyzją i niezastąpiony (R4)');
END;

CREATE TRIGGER transakcja_umowy_zmieniona_platnosc_na_dopuszczony_rachunek
BEFORE UPDATE OF rachunek_id, wplacono ON transakcja_umowy_platnosci
WHEN NEW.rachunek_id IS NOT NULL
  AND (NEW.rachunek_id IS NOT OLD.rachunek_id OR NEW.wplacono IS NOT NULL)
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_rachunki_dopuszczone
    WHERE id = NEW.rachunek_id
  )
BEGIN
  SELECT RAISE(ABORT, 'płatność wskazuje rachunek dopuszczony decyzją i niezastąpiony (R4)');
END;

-- Warunki płatności podpisują strony z umową; po jej wyjściu z projektu zmienia się już tylko
-- rachunek (nowy, znów dopuszczony) i wpłata. Zapłaconej płatności nie zmienia się wcale.
CREATE TRIGGER transakcja_umowy_platnosc_warunki_niezmienne
BEFORE UPDATE ON transakcja_umowy_platnosci
WHEN OLD.wplacono IS NOT NULL
  OR (
    (
      SELECT stan <> 'projekt'
      FROM transakcja_umowy_wpisy
      WHERE id = OLD.umowa_id
    )
    AND (
      NEW.id IS NOT OLD.id
      OR NEW.umowa_id IS NOT OLD.umowa_id
      OR NEW.rodzaj IS NOT OLD.rodzaj
      OR NEW.charakter IS NOT OLD.charakter
      OR NEW.kwota IS NOT OLD.kwota
      OR NEW.termin IS NOT OLD.termin
      OR NEW.zrodlo IS NOT OLD.zrodlo
      OR NEW.bank IS NOT OLD.bank
      OR NEW.warunki_wyplaty IS NOT OLD.warunki_wyplaty
    )
  )
BEGIN
  SELECT RAISE(ABORT, 'warunki płatności zmieniają się tylko w projekcie umowy, a zapłaconej płatności nie zmienia się wcale');
END;

CREATE TRIGGER transakcja_umowy_wplata_z_potwierdzeniem_bankowym
BEFORE UPDATE OF potwierdzenie_dokument_id ON transakcja_umowy_platnosci
WHEN NEW.potwierdzenie_dokument_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_dokumenty_wpisy
    WHERE id = NEW.potwierdzenie_dokument_id
      AND rodzaj = 'potwierdzenie_bankowe'
      AND stan IN ('otrzymany', 'sprawdzony')
  )
BEGIN
  SELECT RAISE(ABORT, 'wpłata wskazuje otrzymane potwierdzenie bankowe');
END;

-- Pierwokupy -------------------------------------------------------------------------------------

-- Wynik przychodzi od kancelarii po zawiadomieniu, więc pierwokup powstaje czekający.
CREATE TRIGGER transakcja_umowy_pierwokup_powstaje_czekajacy
BEFORE INSERT ON transakcja_umowy_pierwokupy
WHEN NEW.wynik <> 'czeka'
BEGIN
  SELECT RAISE(ABORT, 'pierwokup powstaje jako czekający na wynik');
END;

-- Wynik jest ostateczny, a uprawnionego, podstawy i zawiadomienia raz wpisanych się nie zmienia.
CREATE TRIGGER transakcja_umowy_pierwokup_wynik_ostateczny
BEFORE UPDATE ON transakcja_umowy_pierwokupy
WHEN OLD.wynik <> 'czeka'
  OR NEW.id IS NOT OLD.id
  OR NEW.uprawniony IS NOT OLD.uprawniony
  OR NEW.podstawa IS NOT OLD.podstawa
  OR (OLD.zawiadomiono IS NOT NULL AND NEW.zawiadomiono IS NOT OLD.zawiadomiono)
BEGIN
  SELECT RAISE(ABORT, 'wyniku pierwokupu ani raz wpisanego zawiadomienia się nie zmienia');
END;

-- Bez usuwania i zastępowania ---------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER transakcja_umowy_kancelarie_bez_zastepowania
BEFORE INSERT ON transakcja_umowy_kancelarie
WHEN EXISTS (
  SELECT 1
  FROM transakcja_umowy_kancelarie
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER transakcja_umowy_kancelarie_bez_usuwania
BEFORE DELETE ON transakcja_umowy_kancelarie
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER transakcja_umowy_wpisy_bez_zastepowania
BEFORE INSERT ON transakcja_umowy_wpisy
WHEN EXISTS (
  SELECT 1
  FROM transakcja_umowy_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER transakcja_umowy_wpisy_bez_usuwania
BEFORE DELETE ON transakcja_umowy_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER transakcja_umowy_strony_bez_zastepowania
BEFORE INSERT ON transakcja_umowy_strony
WHEN EXISTS (
    SELECT 1
    FROM transakcja_umowy_strony
    WHERE id = NEW.id
  )
  OR EXISTS (
    SELECT 1
    FROM transakcja_umowy_strony
    WHERE umowa_id = NEW.umowa_id
      AND uczestnik_id = NEW.uczestnik_id
  )
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER transakcja_umowy_strony_bez_usuwania
BEFORE DELETE ON transakcja_umowy_strony
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER transakcja_umowy_platnosci_bez_zastepowania
BEFORE INSERT ON transakcja_umowy_platnosci
WHEN EXISTS (
  SELECT 1
  FROM transakcja_umowy_platnosci
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER transakcja_umowy_platnosci_bez_usuwania
BEFORE DELETE ON transakcja_umowy_platnosci
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER transakcja_umowy_pierwokupy_bez_zastepowania
BEFORE INSERT ON transakcja_umowy_pierwokupy
WHEN EXISTS (
  SELECT 1
  FROM transakcja_umowy_pierwokupy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER transakcja_umowy_pierwokupy_bez_usuwania
BEFORE DELETE ON transakcja_umowy_pierwokupy
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;
"#,
];

/// The values of column `pierwokupy.wynik`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PierwokupyWynik {
    Czeka,
    NieWykonano,
    Wykonano,
    Uplynal,
}

impl PierwokupyWynik {
    /// Every value, in the order of the catalog.
    pub const ALL: &[PierwokupyWynik] = &[
        PierwokupyWynik::Czeka,
        PierwokupyWynik::NieWykonano,
        PierwokupyWynik::Wykonano,
        PierwokupyWynik::Uplynal,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            PierwokupyWynik::Czeka => "czeka",
            PierwokupyWynik::NieWykonano => "nie_wykonano",
            PierwokupyWynik::Wykonano => "wykonano",
            PierwokupyWynik::Uplynal => "uplynal",
        }
    }
}

impl xmeta_room::FromSqlValue for PierwokupyWynik {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "czeka" => Ok(PierwokupyWynik::Czeka),
                    "nie_wykonano" => Ok(PierwokupyWynik::NieWykonano),
                    "wykonano" => Ok(PierwokupyWynik::Wykonano),
                    "uplynal" => Ok(PierwokupyWynik::Uplynal),
                    other => Err(format!("{other} is no value of PierwokupyWynik")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for PierwokupyWynik {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `platnosci.rodzaj`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlatnosciRodzaj {
    OplataRezerwacyjna,
    Zadatek,
    Zaliczka,
    Cena,
    SplataHipoteki,
}

impl PlatnosciRodzaj {
    /// Every value, in the order of the catalog.
    pub const ALL: &[PlatnosciRodzaj] = &[
        PlatnosciRodzaj::OplataRezerwacyjna,
        PlatnosciRodzaj::Zadatek,
        PlatnosciRodzaj::Zaliczka,
        PlatnosciRodzaj::Cena,
        PlatnosciRodzaj::SplataHipoteki,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            PlatnosciRodzaj::OplataRezerwacyjna => "oplata_rezerwacyjna",
            PlatnosciRodzaj::Zadatek => "zadatek",
            PlatnosciRodzaj::Zaliczka => "zaliczka",
            PlatnosciRodzaj::Cena => "cena",
            PlatnosciRodzaj::SplataHipoteki => "splata_hipoteki",
        }
    }
}

impl xmeta_room::FromSqlValue for PlatnosciRodzaj {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "oplata_rezerwacyjna" => Ok(PlatnosciRodzaj::OplataRezerwacyjna),
                    "zadatek" => Ok(PlatnosciRodzaj::Zadatek),
                    "zaliczka" => Ok(PlatnosciRodzaj::Zaliczka),
                    "cena" => Ok(PlatnosciRodzaj::Cena),
                    "splata_hipoteki" => Ok(PlatnosciRodzaj::SplataHipoteki),
                    other => Err(format!("{other} is no value of PlatnosciRodzaj")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for PlatnosciRodzaj {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `platnosci.zrodlo`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlatnosciZrodlo {
    Wlasne,
    Kredyt,
}

impl PlatnosciZrodlo {
    /// Every value, in the order of the catalog.
    pub const ALL: &[PlatnosciZrodlo] = &[
        PlatnosciZrodlo::Wlasne,
        PlatnosciZrodlo::Kredyt,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            PlatnosciZrodlo::Wlasne => "wlasne",
            PlatnosciZrodlo::Kredyt => "kredyt",
        }
    }
}

impl xmeta_room::FromSqlValue for PlatnosciZrodlo {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "wlasne" => Ok(PlatnosciZrodlo::Wlasne),
                    "kredyt" => Ok(PlatnosciZrodlo::Kredyt),
                    other => Err(format!("{other} is no value of PlatnosciZrodlo")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for PlatnosciZrodlo {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `strony.strona`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StronyStrona {
    Sprzedajacy,
    Kupujacy,
    Malzonek,
}

impl StronyStrona {
    /// Every value, in the order of the catalog.
    pub const ALL: &[StronyStrona] = &[
        StronyStrona::Sprzedajacy,
        StronyStrona::Kupujacy,
        StronyStrona::Malzonek,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            StronyStrona::Sprzedajacy => "sprzedajacy",
            StronyStrona::Kupujacy => "kupujacy",
            StronyStrona::Malzonek => "malzonek",
        }
    }
}

impl xmeta_room::FromSqlValue for StronyStrona {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "sprzedajacy" => Ok(StronyStrona::Sprzedajacy),
                    "kupujacy" => Ok(StronyStrona::Kupujacy),
                    "malzonek" => Ok(StronyStrona::Malzonek),
                    other => Err(format!("{other} is no value of StronyStrona")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for StronyStrona {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wpisy.rodzaj`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyRodzaj {
    Rezerwacyjna,
    Przedwstepna,
    Warunkowa,
    Przenoszaca,
}

impl WpisyRodzaj {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyRodzaj] = &[
        WpisyRodzaj::Rezerwacyjna,
        WpisyRodzaj::Przedwstepna,
        WpisyRodzaj::Warunkowa,
        WpisyRodzaj::Przenoszaca,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyRodzaj::Rezerwacyjna => "rezerwacyjna",
            WpisyRodzaj::Przedwstepna => "przedwstepna",
            WpisyRodzaj::Warunkowa => "warunkowa",
            WpisyRodzaj::Przenoszaca => "przenoszaca",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyRodzaj {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "rezerwacyjna" => Ok(WpisyRodzaj::Rezerwacyjna),
                    "przedwstepna" => Ok(WpisyRodzaj::Przedwstepna),
                    "warunkowa" => Ok(WpisyRodzaj::Warunkowa),
                    "przenoszaca" => Ok(WpisyRodzaj::Przenoszaca),
                    other => Err(format!("{other} is no value of WpisyRodzaj")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyRodzaj {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wpisy.forma`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyForma {
    Pisemna,
    AktNotarialny,
}

impl WpisyForma {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyForma] = &[WpisyForma::Pisemna, WpisyForma::AktNotarialny];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyForma::Pisemna => "pisemna",
            WpisyForma::AktNotarialny => "akt_notarialny",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyForma {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "pisemna" => Ok(WpisyForma::Pisemna),
                    "akt_notarialny" => Ok(WpisyForma::AktNotarialny),
                    other => Err(format!("{other} is no value of WpisyForma")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyForma {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `wpisy.stan`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyStan {
    Projekt,
    Zawarta,
    NieDoszla,
    Rozwiazana,
}

impl WpisyStan {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyStan] = &[
        WpisyStan::Projekt,
        WpisyStan::Zawarta,
        WpisyStan::NieDoszla,
        WpisyStan::Rozwiazana,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyStan::Projekt => "projekt",
            WpisyStan::Zawarta => "zawarta",
            WpisyStan::NieDoszla => "nie_doszla",
            WpisyStan::Rozwiazana => "rozwiazana",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyStan {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "projekt" => Ok(WpisyStan::Projekt),
                    "zawarta" => Ok(WpisyStan::Zawarta),
                    "nie_doszla" => Ok(WpisyStan::NieDoszla),
                    "rozwiazana" => Ok(WpisyStan::Rozwiazana),
                    other => Err(format!("{other} is no value of WpisyStan")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisyStan {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of field `wynik` of `odnotuj_wynik_pierwokupu`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Wynik {
    NieWykonano,
    Wykonano,
    Uplynal,
}

impl Wynik {
    /// Every value, in the order of the catalog.
    pub const ALL: &[Wynik] = &[Wynik::NieWykonano, Wynik::Wykonano, Wynik::Uplynal];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            Wynik::NieWykonano => "nie_wykonano",
            Wynik::Wykonano => "wykonano",
            Wynik::Uplynal => "uplynal",
        }
    }
}

impl xmeta_room::FromSqlValue for Wynik {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "nie_wykonano" => Ok(Wynik::NieWykonano),
                    "wykonano" => Ok(Wynik::Wykonano),
                    "uplynal" => Ok(Wynik::Uplynal),
                    other => Err(format!("{other} is no value of Wynik")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for Wynik {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// A row of `kancelarie`.
#[derive(Debug, Clone, PartialEq)]
pub struct KancelarieRow {
    pub id: String,
    pub nazwa: String,
    pub adres: Option<String>,
    pub notariusz: Option<String>,
    pub xnotariusz: bool,
    pub otwarcie_decyzja_id: Option<String>,
    pub pokoj_odnosnik: Option<String>,
    pub od: String,
    pub r#do: Option<String>,
    pub powod_zmiany: Option<String>,
}

impl KancelarieRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(KancelarieRow {
            id: row.get("id")?,
            nazwa: row.get("nazwa")?,
            adres: row.get("adres")?,
            notariusz: row.get("notariusz")?,
            xnotariusz: row.get("xnotariusz")?,
            otwarcie_decyzja_id: row.get("otwarcie_decyzja_id")?,
            pokoj_odnosnik: row.get("pokoj_odnosnik")?,
            od: row.get("od")?,
            r#do: row.get("do")?,
            powod_zmiany: row.get("powod_zmiany")?,
        })
    }
}

/// Kancelarie transakcji w kolejności wpisania; bieżąca nie ma daty końca.
pub fn kancelarie(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<KancelarieRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM transakcja_umowy_kancelarie
ORDER BY od, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(KancelarieRow::from_row).collect()
}

/// The parameters of `wpisz_kancelarie`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpiszKancelarieParams {
    pub id: String,
    pub nazwa: String,
    pub adres: Option<String>,
    pub notariusz: Option<String>,
    pub xnotariusz: bool,
    pub od: String,
}

/// Kancelaria, którą wybrały strony; identyfikator (UUID) nadaje aplikacja, bo wskazuje go decyzja
/// o otwarciu pokoju wymiany.
pub fn wpisz_kancelarie(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszKancelarieParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO transakcja_umowy_kancelarie (id, nazwa, adres, notariusz, xnotariusz, od)
VALUES (:id, :nazwa, :adres, :notariusz, :xnotariusz, :od)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("nazwa", xmeta_room::ToSqlValue::to_sql_value(&params.nazwa)),
                    ("adres", xmeta_room::ToSqlValue::to_sql_value(&params.adres)),
                    (
                        "notariusz",
                        xmeta_room::ToSqlValue::to_sql_value(&params.notariusz),
                    ),
                    (
                        "xnotariusz",
                        xmeta_room::ToSqlValue::to_sql_value(&params.xnotariusz),
                    ),
                    ("od", xmeta_room::ToSqlValue::to_sql_value(&params.od)),
                ]),
            },
        ],
    )
}

/// The parameters of `otworz_pokoj_wymiany`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OtworzPokojWymianyParams {
    pub otwarcie_decyzja_id: String,
    pub pokoj_odnosnik: String,
    pub id: String,
}

/// Pokój wymiany z kancelarią w XNotariuszu, otwarty za zatwierdzoną decyzją inna, która wskazuje
/// tę kancelarię.
pub fn otworz_pokoj_wymiany(
    room: &impl xmeta_room::MetaRoom,
    params: &OtworzPokojWymianyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE transakcja_umowy_kancelarie
SET otwarcie_decyzja_id = :otwarcie_decyzja_id,
  pokoj_odnosnik = :pokoj_odnosnik
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "otwarcie_decyzja_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.otwarcie_decyzja_id),
                    ),
                    (
                        "pokoj_odnosnik",
                        xmeta_room::ToSqlValue::to_sql_value(&params.pokoj_odnosnik),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `zakoncz_kancelarie`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZakonczKancelarieParams {
    pub r#do: String,
    pub powod_zmiany: String,
    pub id: String,
}

/// Strony zmieniają kancelarię: bieżąca dostaje datę końca i powód, a nowa to nowy wpis.
pub fn zakoncz_kancelarie(
    room: &impl xmeta_room::MetaRoom,
    params: &ZakonczKancelarieParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE transakcja_umowy_kancelarie
SET do = :do,
  powod_zmiany = :powod_zmiany
WHERE id = :id"#,
                parameters: Vec::from([
                    ("do", xmeta_room::ToSqlValue::to_sql_value(&params.r#do)),
                    (
                        "powod_zmiany",
                        xmeta_room::ToSqlValue::to_sql_value(&params.powod_zmiany),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// A row of `pierwokupy`.
#[derive(Debug, Clone, PartialEq)]
pub struct PierwokupyRow {
    pub id: String,
    pub uprawniony: String,
    pub podstawa: String,
    pub zawiadomiono: Option<String>,
    pub wynik: PierwokupyWynik,
    pub zrodlo_wplyw_id: Option<String>,
}

impl PierwokupyRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(PierwokupyRow {
            id: row.get("id")?,
            uprawniony: row.get("uprawniony")?,
            podstawa: row.get("podstawa")?,
            zawiadomiono: row.get("zawiadomiono")?,
            wynik: row.get("wynik")?,
            zrodlo_wplyw_id: row.get("zrodlo_wplyw_id")?,
        })
    }
}

/// Prawa pierwokupu z wynikiem; czekające mają wynik czeka.
pub fn pierwokupy(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<PierwokupyRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM transakcja_umowy_pierwokupy
ORDER BY rowid"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(PierwokupyRow::from_row).collect()
}

/// The parameters of `wpisz_pierwokup`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WpiszPierwokupParams {
    pub id: String,
    pub uprawniony: String,
    pub podstawa: String,
    pub zawiadomiono: Option<String>,
}

/// Uprawniony z pierwokupu i podstawa prawna, przed umową przenoszącą.
pub fn wpisz_pierwokup(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszPierwokupParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO transakcja_umowy_pierwokupy (id, uprawniony, podstawa, zawiadomiono, wynik)
VALUES (:id, :uprawniony, :podstawa, :zawiadomiono, 'czeka')"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    (
                        "uprawniony",
                        xmeta_room::ToSqlValue::to_sql_value(&params.uprawniony),
                    ),
                    ("podstawa", xmeta_room::ToSqlValue::to_sql_value(&params.podstawa)),
                    (
                        "zawiadomiono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zawiadomiono),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_zawiadomienie_o_pierwokupie`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujZawiadomienieOPierwokupieParams {
    pub zawiadomiono: String,
    pub id: String,
}

/// Kancelaria zawiadomiła uprawnionego o umowie warunkowej; od tego dnia biegnie termin.
pub fn odnotuj_zawiadomienie_o_pierwokupie(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujZawiadomienieOPierwokupieParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE transakcja_umowy_pierwokupy
SET zawiadomiono = :zawiadomiono
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "zawiadomiono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zawiadomiono),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_wynik_pierwokupu`.
#[derive(Debug, Clone, PartialEq)]
pub struct OdnotujWynikPierwokupuParams {
    pub wynik: Wynik,
    pub zrodlo_wplyw_id: String,
    pub id: String,
}

/// Wynik z wiadomości kancelarii: uprawniony nie wykonał prawa, wykonał je albo termin upłynął.
pub fn odnotuj_wynik_pierwokupu(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujWynikPierwokupuParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE transakcja_umowy_pierwokupy
SET wynik = :wynik,
  zrodlo_wplyw_id = :zrodlo_wplyw_id
WHERE id = :id"#,
                parameters: Vec::from([
                    ("wynik", xmeta_room::ToSqlValue::to_sql_value(&params.wynik)),
                    (
                        "zrodlo_wplyw_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.zrodlo_wplyw_id),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// A row of `platnosci_transakcji`.
#[derive(Debug, Clone, PartialEq)]
pub struct PlatnosciTransakcjiRow {
    pub id: String,
    pub umowa_id: String,
    pub rodzaj: PlatnosciRodzaj,
    pub charakter: Option<String>,
    pub kwota: i64,
    pub termin: Option<String>,
    pub zrodlo: PlatnosciZrodlo,
    pub bank: Option<String>,
    pub warunki_wyplaty: Option<String>,
    pub rachunek_id: Option<String>,
    pub wplacono: Option<String>,
    pub potwierdzenie_dokument_id: Option<String>,
}

impl PlatnosciTransakcjiRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(PlatnosciTransakcjiRow {
            id: row.get("id")?,
            umowa_id: row.get("umowa_id")?,
            rodzaj: row.get("rodzaj")?,
            charakter: row.get("charakter")?,
            kwota: row.get("kwota")?,
            termin: row.get("termin")?,
            zrodlo: row.get("zrodlo")?,
            bank: row.get("bank")?,
            warunki_wyplaty: row.get("warunki_wyplaty")?,
            rachunek_id: row.get("rachunek_id")?,
            wplacono: row.get("wplacono")?,
            potwierdzenie_dokument_id: row.get("potwierdzenie_dokument_id")?,
        })
    }
}

/// Płatności z umów transakcji w kolejności wpisania.
pub fn platnosci_transakcji(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<PlatnosciTransakcjiRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM transakcja_umowy_platnosci
ORDER BY rowid"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(PlatnosciTransakcjiRow::from_row).collect()
}

/// The parameters of `wpisz_platnosc_transakcji`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszPlatnoscTransakcjiParams {
    pub id: String,
    pub umowa_id: String,
    pub rodzaj: PlatnosciRodzaj,
    pub charakter: Option<String>,
    pub kwota: i64,
    pub termin: Option<String>,
    pub zrodlo: PlatnosciZrodlo,
    pub bank: Option<String>,
    pub warunki_wyplaty: Option<String>,
    pub rachunek_id: Option<String>,
}

/// Opłata rezerwacyjna, zadatek, zaliczka, cena albo spłata hipoteki z projektu umowy. Przy
/// kredycie także bank i warunki wypłaty. Identyfikator (UUID) nadaje aplikacja.
pub fn wpisz_platnosc_transakcji(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszPlatnoscTransakcjiParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO transakcja_umowy_platnosci (id, umowa_id, rodzaj, charakter, kwota, termin, zrodlo,
  bank, warunki_wyplaty, rachunek_id)
VALUES (:id, :umowa_id, :rodzaj, :charakter, :kwota, :termin, :zrodlo, :bank, :warunki_wyplaty,
  :rachunek_id)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("umowa_id", xmeta_room::ToSqlValue::to_sql_value(&params.umowa_id)),
                    ("rodzaj", xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj)),
                    (
                        "charakter",
                        xmeta_room::ToSqlValue::to_sql_value(&params.charakter),
                    ),
                    ("kwota", xmeta_room::ToSqlValue::to_sql_value(&params.kwota)),
                    ("termin", xmeta_room::ToSqlValue::to_sql_value(&params.termin)),
                    ("zrodlo", xmeta_room::ToSqlValue::to_sql_value(&params.zrodlo)),
                    ("bank", xmeta_room::ToSqlValue::to_sql_value(&params.bank)),
                    (
                        "warunki_wyplaty",
                        xmeta_room::ToSqlValue::to_sql_value(&params.warunki_wyplaty),
                    ),
                    (
                        "rachunek_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.rachunek_id),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `zmien_platnosc_transakcji`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZmienPlatnoscTransakcjiParams {
    pub charakter: Option<String>,
    pub kwota: i64,
    pub termin: Option<String>,
    pub zrodlo: PlatnosciZrodlo,
    pub bank: Option<String>,
    pub warunki_wyplaty: Option<String>,
    pub id: String,
}

/// Poprawka warunków płatności, dopóki umowa jest projektem.
pub fn zmien_platnosc_transakcji(
    room: &impl xmeta_room::MetaRoom,
    params: &ZmienPlatnoscTransakcjiParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE transakcja_umowy_platnosci
SET charakter = :charakter,
  kwota = :kwota,
  termin = :termin,
  zrodlo = :zrodlo,
  bank = :bank,
  warunki_wyplaty = :warunki_wyplaty
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "charakter",
                        xmeta_room::ToSqlValue::to_sql_value(&params.charakter),
                    ),
                    ("kwota", xmeta_room::ToSqlValue::to_sql_value(&params.kwota)),
                    ("termin", xmeta_room::ToSqlValue::to_sql_value(&params.termin)),
                    ("zrodlo", xmeta_room::ToSqlValue::to_sql_value(&params.zrodlo)),
                    ("bank", xmeta_room::ToSqlValue::to_sql_value(&params.bank)),
                    (
                        "warunki_wyplaty",
                        xmeta_room::ToSqlValue::to_sql_value(&params.warunki_wyplaty),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `wskaz_rachunek_platnosci`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WskazRachunekPlatnosciParams {
    pub rachunek_id: String,
    pub id: String,
}

/// Rachunek, na który ma trafić wpłata; dopuszczony decyzją i niezastąpiony (R4).
pub fn wskaz_rachunek_platnosci(
    room: &impl xmeta_room::MetaRoom,
    params: &WskazRachunekPlatnosciParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE transakcja_umowy_platnosci
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

/// The parameters of `odnotuj_wplate_platnosci`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujWplatePlatnosciParams {
    pub wplacono: String,
    pub potwierdzenie_dokument_id: String,
    pub id: String,
}

/// Wpłata doszła: chwila i potwierdzenie z banku.
pub fn odnotuj_wplate_platnosci(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujWplatePlatnosciParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE transakcja_umowy_platnosci
SET wplacono = :wplacono,
  potwierdzenie_dokument_id = :potwierdzenie_dokument_id
WHERE id = :id"#,
                parameters: Vec::from([
                    ("wplacono", xmeta_room::ToSqlValue::to_sql_value(&params.wplacono)),
                    (
                        "potwierdzenie_dokument_id",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.potwierdzenie_dokument_id,
                        ),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// A row of `strony_umow_transakcji`.
#[derive(Debug, Clone, PartialEq)]
pub struct StronyUmowTransakcjiRow {
    pub id: i64,
    pub umowa_id: String,
    pub uczestnik_id: String,
    pub strona: StronyStrona,
    pub umocowanie_id: Option<String>,
}

impl StronyUmowTransakcjiRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(StronyUmowTransakcjiRow {
            id: row.get("id")?,
            umowa_id: row.get("umowa_id")?,
            uczestnik_id: row.get("uczestnik_id")?,
            strona: row.get("strona")?,
            umocowanie_id: row.get("umocowanie_id")?,
        })
    }
}

/// Strony wszystkich umów transakcji z umocowaniem pełnomocnika, jeśli jest.
pub fn strony_umow_transakcji(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<StronyUmowTransakcjiRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM transakcja_umowy_strony
ORDER BY umowa_id, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(StronyUmowTransakcjiRow::from_row).collect()
}

/// The parameters of `dopisz_strone_umowy_transakcji`.
#[derive(Debug, Clone, PartialEq)]
pub struct DopiszStroneUmowyTransakcjiParams {
    pub umowa_id: String,
    pub uczestnik_id: String,
    pub strona: StronyStrona,
    pub umocowanie_id: Option<String>,
}

/// Sprzedający, kupujący albo małżonek, który wyraża zgodę; przy pełnomocniku jego umocowanie.
pub fn dopisz_strone_umowy_transakcji(
    room: &impl xmeta_room::MetaRoom,
    params: &DopiszStroneUmowyTransakcjiParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO transakcja_umowy_strony (umowa_id, uczestnik_id, strona, umocowanie_id)
VALUES (:umowa_id, :uczestnik_id, :strona, :umocowanie_id)"#,
                parameters: Vec::from([
                    ("umowa_id", xmeta_room::ToSqlValue::to_sql_value(&params.umowa_id)),
                    (
                        "uczestnik_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.uczestnik_id),
                    ),
                    ("strona", xmeta_room::ToSqlValue::to_sql_value(&params.strona)),
                    (
                        "umocowanie_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.umocowanie_id),
                    ),
                ]),
            },
        ],
    )
}

/// A row of `umowy_transakcji`.
#[derive(Debug, Clone, PartialEq)]
pub struct UmowyTransakcjiRow {
    pub id: String,
    pub rodzaj: WpisyRodzaj,
    pub forma: WpisyForma,
    pub stan: WpisyStan,
    pub oparta_na_id: Option<String>,
    pub koniec_powod: Option<String>,
    pub kancelaria_id: Option<String>,
    pub projekt_dokument_id: Option<String>,
    pub dokument_id: Option<String>,
    pub zawarto: Option<String>,
    pub repertorium: Option<String>,
    pub cena: Option<i64>,
    pub termin_przyrzeczonej: Option<String>,
    pub termin_zaplaty: Option<String>,
    pub termin_wydania: Option<String>,
    pub warunki: Option<String>,
    pub wniosek_kw_zlozono: Option<String>,
    pub wpis_kw: Option<String>,
}

impl UmowyTransakcjiRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(UmowyTransakcjiRow {
            id: row.get("id")?,
            rodzaj: row.get("rodzaj")?,
            forma: row.get("forma")?,
            stan: row.get("stan")?,
            oparta_na_id: row.get("oparta_na_id")?,
            koniec_powod: row.get("koniec_powod")?,
            kancelaria_id: row.get("kancelaria_id")?,
            projekt_dokument_id: row.get("projekt_dokument_id")?,
            dokument_id: row.get("dokument_id")?,
            zawarto: row.get("zawarto")?,
            repertorium: row.get("repertorium")?,
            cena: row.get("cena")?,
            termin_przyrzeczonej: row.get("termin_przyrzeczonej")?,
            termin_zaplaty: row.get("termin_zaplaty")?,
            termin_wydania: row.get("termin_wydania")?,
            warunki: row.get("warunki")?,
            wniosek_kw_zlozono: row.get("wniosek_kw_zlozono")?,
            wpis_kw: row.get("wpis_kw")?,
        })
    }
}

/// Umowy stron w kolejności wpisania.
pub fn umowy_transakcji(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<UmowyTransakcjiRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM transakcja_umowy_wpisy
ORDER BY rowid"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&statement)?;
    rows.iter().map(UmowyTransakcjiRow::from_row).collect()
}

/// The parameters of `przygotuj_umowe_transakcji`.
#[derive(Debug, Clone, PartialEq)]
pub struct PrzygotujUmoweTransakcjiParams {
    pub id: String,
    pub rodzaj: WpisyRodzaj,
    pub forma: WpisyForma,
    pub oparta_na_id: Option<String>,
    pub kancelaria_id: Option<String>,
    pub projekt_dokument_id: Option<String>,
    pub cena: Option<i64>,
    pub termin_przyrzeczonej: Option<String>,
    pub termin_zaplaty: Option<String>,
    pub termin_wydania: Option<String>,
    pub warunki: Option<String>,
}

/// Projekt umowy rezerwacyjnej, przedwstępnej, warunkowej albo przenoszącej; identyfikator (UUID)
/// nadaje aplikacja, bo wskazują go strony, płatności i analizy.
pub fn przygotuj_umowe_transakcji(
    room: &impl xmeta_room::MetaRoom,
    params: &PrzygotujUmoweTransakcjiParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO transakcja_umowy_wpisy (id, rodzaj, forma, oparta_na_id, kancelaria_id,
  projekt_dokument_id, cena, termin_przyrzeczonej, termin_zaplaty, termin_wydania, warunki, stan)
VALUES (:id, :rodzaj, :forma, :oparta_na_id, :kancelaria_id, :projekt_dokument_id, :cena,
  :termin_przyrzeczonej, :termin_zaplaty, :termin_wydania, :warunki, 'projekt')"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("rodzaj", xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj)),
                    ("forma", xmeta_room::ToSqlValue::to_sql_value(&params.forma)),
                    (
                        "oparta_na_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.oparta_na_id),
                    ),
                    (
                        "kancelaria_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.kancelaria_id),
                    ),
                    (
                        "projekt_dokument_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.projekt_dokument_id),
                    ),
                    ("cena", xmeta_room::ToSqlValue::to_sql_value(&params.cena)),
                    (
                        "termin_przyrzeczonej",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.termin_przyrzeczonej,
                        ),
                    ),
                    (
                        "termin_zaplaty",
                        xmeta_room::ToSqlValue::to_sql_value(&params.termin_zaplaty),
                    ),
                    (
                        "termin_wydania",
                        xmeta_room::ToSqlValue::to_sql_value(&params.termin_wydania),
                    ),
                    ("warunki", xmeta_room::ToSqlValue::to_sql_value(&params.warunki)),
                ]),
            },
        ],
    )
}

/// The parameters of `zmien_projekt_umowy_transakcji`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZmienProjektUmowyTransakcjiParams {
    pub kancelaria_id: Option<String>,
    pub projekt_dokument_id: Option<String>,
    pub cena: Option<i64>,
    pub termin_przyrzeczonej: Option<String>,
    pub termin_zaplaty: Option<String>,
    pub termin_wydania: Option<String>,
    pub warunki: Option<String>,
    pub id: String,
}

/// Nowa wersja projektu: inny dokument projektu albo inne warunki. Nowy dokument wymaga nowej
/// analizy, zanim strony podpiszą umowę.
pub fn zmien_projekt_umowy_transakcji(
    room: &impl xmeta_room::MetaRoom,
    params: &ZmienProjektUmowyTransakcjiParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE transakcja_umowy_wpisy
SET kancelaria_id = :kancelaria_id,
  projekt_dokument_id = :projekt_dokument_id,
  cena = :cena,
  termin_przyrzeczonej = :termin_przyrzeczonej,
  termin_zaplaty = :termin_zaplaty,
  termin_wydania = :termin_wydania,
  warunki = :warunki
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "kancelaria_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.kancelaria_id),
                    ),
                    (
                        "projekt_dokument_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.projekt_dokument_id),
                    ),
                    ("cena", xmeta_room::ToSqlValue::to_sql_value(&params.cena)),
                    (
                        "termin_przyrzeczonej",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.termin_przyrzeczonej,
                        ),
                    ),
                    (
                        "termin_zaplaty",
                        xmeta_room::ToSqlValue::to_sql_value(&params.termin_zaplaty),
                    ),
                    (
                        "termin_wydania",
                        xmeta_room::ToSqlValue::to_sql_value(&params.termin_wydania),
                    ),
                    ("warunki", xmeta_room::ToSqlValue::to_sql_value(&params.warunki)),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_zawarcie_umowy_transakcji`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujZawarcieUmowyTransakcjiParams {
    pub dokument_id: String,
    pub zawarto: String,
    pub repertorium: Option<String>,
    pub id: String,
}

/// Strony podpisały umowę: podpisany egzemplarz albo wypis aktu, chwila i przy akcie numer
/// repertorium.
pub fn odnotuj_zawarcie_umowy_transakcji(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujZawarcieUmowyTransakcjiParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE transakcja_umowy_wpisy
SET stan = 'zawarta',
  dokument_id = :dokument_id,
  zawarto = :zawarto,
  repertorium = :repertorium
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "dokument_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dokument_id),
                    ),
                    ("zawarto", xmeta_room::ToSqlValue::to_sql_value(&params.zawarto)),
                    (
                        "repertorium",
                        xmeta_room::ToSqlValue::to_sql_value(&params.repertorium),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_ze_umowa_transakcji_nie_doszla`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujZeUmowaTransakcjiNieDoszlaParams {
    pub koniec_powod: String,
    pub id: String,
}

/// Projekt, którego strony nie podpiszą, np. bo bank odmówił kredytu.
pub fn odnotuj_ze_umowa_transakcji_nie_doszla(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujZeUmowaTransakcjiNieDoszlaParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE transakcja_umowy_wpisy
SET stan = 'nie_doszla',
  koniec_powod = :koniec_powod
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "koniec_powod",
                        xmeta_room::ToSqlValue::to_sql_value(&params.koniec_powod),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `rozwiaz_umowe_transakcji`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct RozwiazUmoweTransakcjiParams {
    pub koniec_powod: String,
    pub id: String,
}

/// Zawarta umowa przestaje wiązać, np. po odstąpieniu z zadatkiem (art. 394 KC).
pub fn rozwiaz_umowe_transakcji(
    room: &impl xmeta_room::MetaRoom,
    params: &RozwiazUmoweTransakcjiParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE transakcja_umowy_wpisy
SET stan = 'rozwiazana',
  koniec_powod = :koniec_powod
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "koniec_powod",
                        xmeta_room::ToSqlValue::to_sql_value(&params.koniec_powod),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_wniosek_do_ksiegi`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujWniosekDoKsiegiParams {
    pub wniosek_kw_zlozono: String,
    pub id: String,
}

/// Notariusz złożył wniosek o wpis kupującego do księgi wieczystej (art. 92 § 4 PoN).
pub fn odnotuj_wniosek_do_ksiegi(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujWniosekDoKsiegiParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE transakcja_umowy_wpisy
SET wniosek_kw_zlozono = :wniosek_kw_zlozono
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "wniosek_kw_zlozono",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wniosek_kw_zlozono),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_wpis_do_ksiegi`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujWpisDoKsiegiParams {
    pub wpis_kw: String,
    pub id: String,
}

/// Sąd wpisał kupującego do księgi; Koordynator informuje o tym strony.
pub fn odnotuj_wpis_do_ksiegi(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujWpisDoKsiegiParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE transakcja_umowy_wpisy
SET wpis_kw = :wpis_kw
WHERE id = :id"#,
                parameters: Vec::from([
                    ("wpis_kw", xmeta_room::ToSqlValue::to_sql_value(&params.wpis_kw)),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}
