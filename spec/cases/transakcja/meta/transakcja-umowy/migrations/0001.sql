-- Umowy stron transakcji: kancelaria notarialna, umowy rezerwacyjna, przedwstępna, warunkowa
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
