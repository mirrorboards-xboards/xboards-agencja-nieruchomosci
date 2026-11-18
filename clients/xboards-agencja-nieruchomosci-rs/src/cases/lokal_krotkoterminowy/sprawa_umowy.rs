//! The kind `sprawa-umowy` of the room `cases/lokal_krotkoterminowy`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind: its name, the kinds it requires and its migrations in the order of their
/// numbers.
pub const KIND: xmeta_room::Kind = xmeta_room::Kind {
    name: "sprawa-umowy",
    requires: &[
        &super::sprawa_dokumenty::KIND,
        &super::sprawa_korespondencja::KIND,
        &super::sprawa_metryka::KIND,
        &super::sprawa_nieruchomosci::KIND,
        &super::sprawa_sprawdzenia::KIND,
        &super::sprawa_uczestnicy::KIND,
        &super::sprawa_zgody::KIND,
    ],
    migrations: &[
        r#"-- Umowy klienta z biurem: pośrednictwa i o zarządzanie nieruchomością, ze stronami, zakresem
-- czynności i zdarzeniami (aneks, polisa, żądanie, odstąpienie, wypowiedzenie, wynagrodzenie).
-- Umowę przygotowuje agent sprawy, a podpisuje człowiek. Typ sprawy dodaje tylko swoje warunki,
-- np. widoczność oferty albo widełki cen najmu (D12).
--
-- Przed podpisem każda strona ma aktualne sprawdzenie tożsamości „zgodne” (R7, art. 39 AML) i nie
-- stoi znacznik wątpliwości. Zawarta umowa ma podpisany dokument i kopię polisy pośrednika
-- (art. 181 i 186 ust. 3a ugn).
--
-- Konsument, który zawarł umowę na odległość albo poza lokalem biura, może od niej odstąpić
-- (art. 27 upk). Biuro zaczyna pracę przed końcem terminu tylko na jego wyraźne żądanie (R8);
-- przy umowie poza lokalem żądanie jest na trwałym nośniku (art. 15 ust. 3 upk). Koniec terminu
-- (odstapienie_do) liczy program. Widok sprawa_umowy_praca_od mówi, od kiedy biuro może pracować
-- według umowy, a zakaz pracy przed tą chwilą pilnują reguły typów na ich danych.
--
-- Rodzaj umowy nie zmienia się od wpisu. Po zawarciu dane zawarcia się nie zmieniają. Warunki
-- zmienia tylko aneks: warunki_od mówi, od kiedy obowiązują bieżące warunki, i przy każdej zmianie
-- wskazuje chwilę najnowszego aneksu. Wyjątkiem jest koniec umowy po wypowiedzeniu albo
-- odstąpieniu: wpisuje się go albo skraca bez aneksu.
-- Polisę zmienia tylko zdarzenie nowa_polisa. Dokumentów, na których stoi zawarta umowa, nie
-- odrzuca się ani nie przypisuje innej sprawie.
CREATE TABLE sprawa_umowy_wpisy (
  id TEXT PRIMARY KEY,
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('posrednictwo', 'zarzadzanie')),
  stan TEXT NOT NULL CHECK (stan IN ('projekt', 'do_podpisu', 'zawarta', 'zakonczona')),
  koniec_powod TEXT,
  forma TEXT CHECK (forma IN ('pisemna', 'elektroniczna')),
  sposob_zawarcia TEXT CHECK (sposob_zawarcia IN ('w_lokalu_biura', 'poza_lokalem', 'na_odleglosc', 'nieumowiona_wizyta')),
  zawarto TEXT,
  -- Człowiek, który podpisał umowę za biuro.
  podpisal_za_biuro TEXT,
  dokument_id TEXT REFERENCES sprawa_dokumenty_wpisy (id),
  polisa_dokument_id TEXT REFERENCES sprawa_dokumenty_wpisy (id),
  nieruchomosc_id TEXT REFERENCES sprawa_nieruchomosci_wpisy (id),
  poczatek TEXT,
  koniec TEXT,
  -- Np. „1 miesiąc ze skutkiem na koniec miesiąca”.
  okres_wypowiedzenia TEXT,
  -- Biuro pośredniczy dla obu stron tej samej transakcji (R1).
  obie_strony INTEGER NOT NULL DEFAULT 0 CHECK (obie_strony IN (0, 1)),
  wylacznosc INTEGER NOT NULL DEFAULT 0 CHECK (wylacznosc IN (0, 1)),
  wynagrodzenie_opis TEXT,
  -- Zdarzenie, od którego biuro zarabia, np. zawarcie umowy przyrzeczonej.
  wynagrodzenie_zdarzenie TEXT,
  -- Od kiedy obowiązują bieżące warunki (koniec, okres wypowiedzenia, wyłączność, wynagrodzenie):
  -- przy zawarciu chwila zawarcia, po aneksie chwila najnowszego aneksu.
  warunki_od TEXT,
  -- Wysyłka, w której biuro poinformowało konsumenta o prawie odstąpienia.
  informacja_o_odstapieniu_wysylka_id TEXT REFERENCES sprawa_korespondencja_wysylki (id),
  -- Ostatni dzień terminu na odstąpienie od umowy (art. 27–29 upk), liczony przez program
  -- z przesunięciem na dzień roboczy (art. 115 KC); tylko przy umowie, od której konsument może
  -- odstąpić (sprawa_umowy_odstapienie_do_przy_prawie_odstapienia).
  odstapienie_do TEXT,

  CONSTRAINT zakonczona_ma_powod CHECK ((stan = 'zakonczona') = (koniec_powod IS NOT NULL)),
  CONSTRAINT koniec_nie_przed_poczatkiem CHECK (koniec IS NULL OR poczatek IS NULL OR koniec >= poczatek),
  CONSTRAINT warunki_od_przy_zawarciu CHECK (
    (zawarto IS NULL) = (warunki_od IS NULL)
    AND (warunki_od IS NULL OR warunki_od >= zawarto)
  ),
  -- Dzień bez godziny: praca_od zaczyna się w dniu po nim.
  CONSTRAINT odstapienie_do_jest_dniem_po_zawarciu CHECK (
    odstapienie_do IS NULL
    OR (zawarto IS NOT NULL AND odstapienie_do = date(odstapienie_do))
  )
) STRICT;

-- Klienci, którzy są stronami umowy z biurem.
CREATE TABLE sprawa_umowy_strony (
  -- Klucz nadany przez bazę jest dodatni; zob. sprawa_umowy_strony_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  umowa_id TEXT NOT NULL REFERENCES sprawa_umowy_wpisy (id),
  uczestnik_id TEXT NOT NULL REFERENCES sprawa_uczestnicy_wpisy (id),

  UNIQUE (umowa_id, uczestnik_id)
) STRICT;

-- Czynności, na które umowa pozwala biuru, nazwane jak w typie sprawy.
CREATE TABLE sprawa_umowy_zakres (
  -- Klucz nadany przez bazę jest dodatni; zob. sprawa_umowy_zakres_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  umowa_id TEXT NOT NULL REFERENCES sprawa_umowy_wpisy (id),
  czynnosc TEXT NOT NULL CHECK (czynnosc <> '' AND czynnosc NOT GLOB '*[^a-z_]*'),

  UNIQUE (umowa_id, czynnosc)
) STRICT;

-- Zdarzenia umowy, każde z dowodem: dokumentem, wysyłką, wpływem albo zgodą. Kartoteka klientów
-- wskazuje zdarzenie odstąpienia, więc ma UUID.
CREATE TABLE sprawa_umowy_zdarzenia (
  id TEXT PRIMARY KEY,
  umowa_id TEXT NOT NULL REFERENCES sprawa_umowy_wpisy (id),
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('aneks', 'nowa_polisa', 'wezwanie_o_polise', 'zadanie_rozpoczecia_pracy', 'odstapienie', 'wypowiedzenie', 'zdarzenie_wynagrodzenia')),
  chwila TEXT NOT NULL,
  opis TEXT,
  dokument_id TEXT REFERENCES sprawa_dokumenty_wpisy (id),
  wysylka_id TEXT REFERENCES sprawa_korespondencja_wysylki (id),
  wplyw_id TEXT REFERENCES sprawa_korespondencja_wplywy (id),
  zgoda_id TEXT REFERENCES sprawa_zgody_wpisy (id),

  CONSTRAINT zdarzenie_ma_dowod CHECK (
    dokument_id IS NOT NULL OR wysylka_id IS NOT NULL OR wplyw_id IS NOT NULL OR zgoda_id IS NOT NULL
  )
) STRICT;

-- Reguły podpisu i zawarcia sprawdzają zmianę stanu, więc umowa nie może powstać od razu dalej.
CREATE TRIGGER sprawa_umowy_powstaje_jako_projekt
BEFORE INSERT ON sprawa_umowy_wpisy
WHEN NEW.stan <> 'projekt'
BEGIN
  SELECT RAISE(ABORT, 'umowa powstaje jako projekt');
END;

-- Od rodzaju zależy, w której teczce umowa leży i które reguły typu ją obejmują, więc rodzaj nie
-- zmienia się od wpisu, także w projekcie. Inny rodzaj to nowy projekt.
CREATE TRIGGER sprawa_umowy_rodzaj_niezmienny
BEFORE UPDATE OF rodzaj ON sprawa_umowy_wpisy
WHEN NEW.rodzaj IS NOT OLD.rodzaj
BEGIN
  SELECT RAISE(ABORT, 'rodzaj umowy się nie zmienia; inny rodzaj to nowy projekt');
END;

-- Projekt idzie do podpisu i może z niego wrócić do poprawki; zawarta umowa się kończy. Kończy
-- się też projekt, którego klient nie podpisze.
CREATE TRIGGER sprawa_umowy_przejscia_stanu
BEFORE UPDATE OF stan ON sprawa_umowy_wpisy
WHEN NEW.stan IS NOT OLD.stan
  AND NOT (
    (OLD.stan = 'projekt' AND NEW.stan IN ('do_podpisu', 'zakonczona'))
    OR (OLD.stan = 'do_podpisu' AND NEW.stan IN ('projekt', 'zawarta', 'zakonczona'))
    OR (OLD.stan = 'zawarta' AND NEW.stan = 'zakonczona')
  )
BEGIN
  SELECT RAISE(ABORT, 'umowa przechodzi: projekt, do podpisu, zawarta, zakończona; do projektu wraca tylko z podpisu');
END;

CREATE TRIGGER sprawa_umowy_do_podpisu_po_sprawdzeniu_stron
BEFORE UPDATE OF stan ON sprawa_umowy_wpisy
WHEN NEW.stan = 'do_podpisu'
  AND OLD.stan = 'projekt'
  AND (
    NOT EXISTS (
      SELECT 1
      FROM sprawa_umowy_strony
      WHERE umowa_id = NEW.id
    )
    OR EXISTS (
      SELECT 1
      FROM sprawa_umowy_strony AS strona
      WHERE strona.umowa_id = NEW.id
        AND NOT EXISTS (
          SELECT 1
          FROM sprawa_sprawdzenia_wpisy AS sprawdzenie
          WHERE sprawdzenie.uczestnik_id = strona.uczestnik_id
            AND sprawdzenie.rodzaj = 'tozsamosc'
            AND sprawdzenie.aktualne = 1
            AND sprawdzenie.wynik = 'zgodne'
        )
    )
  )
BEGIN
  SELECT RAISE(ABORT, 'umowa idzie do podpisu, gdy ma strony, a każda ma aktualne sprawdzenie tożsamości „zgodne”');
END;

CREATE TRIGGER sprawa_umowy_watpliwosc_blokuje_podpis
BEFORE UPDATE OF stan ON sprawa_umowy_wpisy
WHEN NEW.stan = 'do_podpisu'
  AND OLD.stan = 'projekt'
  AND EXISTS (
    SELECT 1
    FROM sprawa_metryka_watpliwosci
    WHERE zdjecie_decyzja_id IS NULL
  )
BEGIN
  SELECT RAISE(ABORT, 'dopóki stoi znacznik wątpliwości, umowa nie idzie do podpisu');
END;

-- Między przekazaniem do podpisu a podpisem sprawdzenie strony mogło stracić aktualność, a Kontroler
-- mógł postawić znacznik, więc zawarcie sprawdza warunki podpisu jeszcze raz.
CREATE TRIGGER sprawa_umowy_zawarcie_po_sprawdzeniu_stron
BEFORE UPDATE OF stan ON sprawa_umowy_wpisy
WHEN NEW.stan = 'zawarta'
  AND OLD.stan = 'do_podpisu'
  AND (
    EXISTS (
      SELECT 1
      FROM sprawa_umowy_strony AS strona
      WHERE strona.umowa_id = NEW.id
        AND NOT EXISTS (
          SELECT 1
          FROM sprawa_sprawdzenia_wpisy AS sprawdzenie
          WHERE sprawdzenie.uczestnik_id = strona.uczestnik_id
            AND sprawdzenie.rodzaj = 'tozsamosc'
            AND sprawdzenie.aktualne = 1
            AND sprawdzenie.wynik = 'zgodne'
        )
    )
    OR EXISTS (
      SELECT 1
      FROM sprawa_metryka_watpliwosci
      WHERE zdjecie_decyzja_id IS NULL
    )
  )
BEGIN
  SELECT RAISE(ABORT, 'umowa zostaje zawarta, gdy każda strona ma aktualne sprawdzenie tożsamości „zgodne” i nie stoi znacznik wątpliwości');
END;

-- Od znacznika konsumenta zależy prawo odstąpienia (art. 27 upk) i reguła oświadczenia
-- odstąpienia. Gdy strona podpisuje albo podpisała umowę, znacznik się nie zmienia.
CREATE TRIGGER sprawa_umowy_konsument_strony_niezmienny
BEFORE UPDATE OF konsument ON sprawa_uczestnicy_wpisy
WHEN NEW.konsument IS NOT OLD.konsument
  AND EXISTS (
    SELECT 1
    FROM sprawa_umowy_strony AS strona
    JOIN sprawa_umowy_wpisy AS umowa ON umowa.id = strona.umowa_id
    WHERE strona.uczestnik_id = NEW.id
      AND (umowa.stan IN ('do_podpisu', 'zawarta') OR umowa.zawarto IS NOT NULL)
  )
BEGIN
  SELECT RAISE(ABORT, 'znacznik konsumenta strony umowy, która wyszła z projektu, się nie zmienia');
END;

-- Sprawdzone strony są stronami umowy, która idzie do podpisu.
CREATE TRIGGER sprawa_umowy_strony_tylko_w_projekcie
BEFORE INSERT ON sprawa_umowy_strony
WHEN (
  SELECT stan <> 'projekt'
  FROM sprawa_umowy_wpisy
  WHERE id = NEW.umowa_id
)
BEGIN
  SELECT RAISE(ABORT, 'strony dopisuje się tylko do projektu umowy');
END;

-- Zakres jest częścią tego, co strony podpisują, więc dopisuje się go tylko do projektu.
CREATE TRIGGER sprawa_umowy_zakres_tylko_w_projekcie
BEFORE INSERT ON sprawa_umowy_zakres
WHEN (
  SELECT stan <> 'projekt'
  FROM sprawa_umowy_wpisy
  WHERE id = NEW.umowa_id
)
BEGIN
  SELECT RAISE(ABORT, 'zakres dopisuje się tylko do projektu umowy');
END;

CREATE TRIGGER sprawa_umowy_strony_bez_zmian
BEFORE UPDATE ON sprawa_umowy_strony
BEGIN
  SELECT RAISE(ABORT, 'strony umowy się nie zmienia; nowa strona to nowy wiersz projektu');
END;

-- Sprawdzane przy każdym zapisie zawartej umowy, więc jej danych zawarcia nie da się potem usunąć.
CREATE TRIGGER sprawa_umowy_zawarta_z_dokumentem_i_polisa
BEFORE UPDATE ON sprawa_umowy_wpisy
WHEN NEW.stan = 'zawarta'
  AND (
    NEW.forma IS NULL
    OR NEW.sposob_zawarcia IS NULL
    OR NEW.zawarto IS NULL
    OR NEW.podpisal_za_biuro IS NULL
    OR NOT EXISTS (
      SELECT 1
      FROM sprawa_dokumenty_wpisy
      WHERE id = NEW.dokument_id
        AND rodzaj = 'umowa_z_biurem'
        AND stan IN ('otrzymany', 'sprawdzony')
    )
    OR NOT EXISTS (
      SELECT 1
      FROM sprawa_dokumenty_wpisy
      WHERE id = NEW.polisa_dokument_id
        AND rodzaj = 'polisa'
        AND stan IN ('otrzymany', 'sprawdzony')
    )
  )
BEGIN
  SELECT RAISE(ABORT, 'umowa zawarta ma podpisany dokument umowy z biurem, kopię polisy, formę i sposób zawarcia');
END;

-- Art. 27 upk: konsument, który zawarł umowę na odległość albo poza lokalem biura (także w czasie
-- nieumówionej wizyty), może od niej odstąpić. Termin biegnie od dnia zawarcia (art. 28 pkt 2)
-- i ma 14 dni, a po nieumówionej wizycie 30 dni (art. 27 ust. 2); dnia zawarcia się nie liczy
-- (art. 111 § 2 KC). Bez informacji o prawie odstąpienia termin jest dłuższy (art. 29), a koniec
-- w dzień wolny przechodzi na następny dzień roboczy (art. 115 KC), więc odstapienie_do jest
-- najwcześniej ostatnim dniem zwykłego terminu. Umowa, od której nie można tak odstąpić, nie ma
-- odstapienie_do. Sprawdzane przy każdym zapisie zawartej umowy; strony i znacznik konsumenta
-- są wtedy już stałe (sprawa_umowy_konsument_strony_niezmienny).
CREATE TRIGGER sprawa_umowy_odstapienie_do_przy_prawie_odstapienia
BEFORE UPDATE ON sprawa_umowy_wpisy
WHEN NEW.stan = 'zawarta'
  AND NOT coalesce(
    CASE
      WHEN NEW.sposob_zawarcia IN ('poza_lokalem', 'na_odleglosc', 'nieumowiona_wizyta')
        AND EXISTS (
          SELECT 1
          FROM sprawa_umowy_strony AS strona
          JOIN sprawa_uczestnicy_wpisy AS uczestnik ON uczestnik.id = strona.uczestnik_id
          WHERE strona.umowa_id = NEW.id
            AND uczestnik.konsument = 1
        )
      THEN NEW.odstapienie_do >= date(
        NEW.zawarto,
        CASE NEW.sposob_zawarcia WHEN 'nieumowiona_wizyta' THEN '+30 days' ELSE '+14 days' END
      )
      ELSE NEW.odstapienie_do IS NULL
    END,
    0
  )
BEGIN
  SELECT RAISE(ABORT, 'umowa z konsumentem zawarta poza lokalem biura albo na odległość ma odstapienie_do nie wcześniej niż koniec terminu z art. 27 upk; inna umowa go nie ma');
END;

-- Od kiedy biuro może pracować według zawartej umowy. Bez prawa odstąpienia od chwili zawarcia.
-- Z prawem odstąpienia od dnia po odstapienie_do, a wcześniej tylko na wyraźne żądanie konsumenta
-- (art. 15 ust. 3 i art. 21 ust. 2 upk, R8): od pierwszego zdarzenia zadanie_rozpoczecia_pracy,
-- ale nie przed zawarciem. Typy porównują z tą chwilą chwilę z własnego wiersza, nie bieżący czas.
-- Czy umowa jeszcze obowiązuje (odstąpienie, koniec), typ sprawdza w jej stanie.
CREATE VIEW sprawa_umowy_praca_od AS
SELECT
  umowa.id AS umowa_id,
  CASE
    WHEN umowa.odstapienie_do IS NULL THEN umowa.zawarto
    WHEN zadanie.pierwsze IS NULL OR zadanie.pierwsze >= date(umowa.odstapienie_do, '+1 day')
      THEN date(umowa.odstapienie_do, '+1 day')
    WHEN zadanie.pierwsze < umowa.zawarto THEN umowa.zawarto
    ELSE zadanie.pierwsze
  END AS od
FROM sprawa_umowy_wpisy AS umowa
LEFT JOIN (
  SELECT
    umowa_id,
    min(chwila) AS pierwsze
  FROM sprawa_umowy_zdarzenia
  WHERE rodzaj = 'zadanie_rozpoczecia_pracy'
  GROUP BY umowa_id
) AS zadanie ON zadanie.umowa_id = umowa.id
WHERE umowa.zawarto IS NOT NULL;

-- Przy zawarciu bieżące warunki obowiązują od chwili zawarcia.
CREATE TRIGGER sprawa_umowy_warunki_od_zawarcia
BEFORE UPDATE OF stan ON sprawa_umowy_wpisy
WHEN NEW.stan = 'zawarta'
  AND OLD.stan <> 'zawarta'
  AND NEW.warunki_od IS NOT NEW.zawarto
BEGIN
  SELECT RAISE(ABORT, 'przy zawarciu warunki umowy obowiązują od chwili zawarcia');
END;

-- Dane zawarcia są tym, co strony podpisały, a reguły odstąpienia i trwałego nośnika czytają
-- sposób zawarcia; po zawarciu się ich nie zmienia. Informację o odstąpieniu biuro daje przed
-- zawarciem (art. 12 upk).
CREATE TRIGGER sprawa_umowy_dane_zawarcia_niezmienne
BEFORE UPDATE ON sprawa_umowy_wpisy
WHEN OLD.stan IN ('zawarta', 'zakonczona')
  AND (
    NEW.id IS NOT OLD.id
    OR NEW.forma IS NOT OLD.forma
    OR NEW.sposob_zawarcia IS NOT OLD.sposob_zawarcia
    OR NEW.zawarto IS NOT OLD.zawarto
    OR NEW.podpisal_za_biuro IS NOT OLD.podpisal_za_biuro
    OR NEW.dokument_id IS NOT OLD.dokument_id
    OR NEW.nieruchomosc_id IS NOT OLD.nieruchomosc_id
    OR NEW.obie_strony IS NOT OLD.obie_strony
    OR NEW.poczatek IS NOT OLD.poczatek
    OR NEW.informacja_o_odstapieniu_wysylka_id IS NOT OLD.informacja_o_odstapieniu_wysylka_id
    OR NEW.odstapienie_do IS NOT OLD.odstapienie_do
  )
BEGIN
  SELECT RAISE(ABORT, 'dane zawarcia umowy się nie zmieniają; po zawarciu zmienia się tylko stan, a warunki i polisę zmienia aneks albo nowa polisa');
END;

-- Warunki zawartej umowy zmienia tylko aneks: warunki_od wskazuje chwilę najnowszego aneksu tej
-- umowy i jest późniejsze niż poprzednie warunki. Porównanie z pustą wartością daje NULL, więc
-- coalesce zamienia je na odmowę. Sam koniec po wypowiedzeniu albo odstąpieniu sprawdza
-- sprawa_umowy_koniec_po_wypowiedzeniu_albo_odstapieniu.
CREATE TRIGGER sprawa_umowy_warunki_zmienia_aneks
BEFORE UPDATE ON sprawa_umowy_wpisy
WHEN OLD.stan IN ('zawarta', 'zakonczona')
  AND (
    NEW.koniec IS NOT OLD.koniec
    OR NEW.okres_wypowiedzenia IS NOT OLD.okres_wypowiedzenia
    OR NEW.wylacznosc IS NOT OLD.wylacznosc
    OR NEW.wynagrodzenie_opis IS NOT OLD.wynagrodzenie_opis
    OR NEW.wynagrodzenie_zdarzenie IS NOT OLD.wynagrodzenie_zdarzenie
    OR NEW.warunki_od IS NOT OLD.warunki_od
  )
  AND NOT (
    NEW.okres_wypowiedzenia IS OLD.okres_wypowiedzenia
    AND NEW.wylacznosc IS OLD.wylacznosc
    AND NEW.wynagrodzenie_opis IS OLD.wynagrodzenie_opis
    AND NEW.wynagrodzenie_zdarzenie IS OLD.wynagrodzenie_zdarzenie
    AND NEW.warunki_od IS OLD.warunki_od
    AND EXISTS (
      SELECT 1
      FROM sprawa_umowy_zdarzenia
      WHERE umowa_id = NEW.id
        AND rodzaj IN ('wypowiedzenie', 'odstapienie')
        AND chwila > OLD.warunki_od
    )
  )
  AND NOT coalesce(
    NEW.warunki_od > OLD.warunki_od
    AND NEW.warunki_od = (
      SELECT max(chwila)
      FROM sprawa_umowy_zdarzenia
      WHERE umowa_id = NEW.id
        AND rodzaj = 'aneks'
    ),
    0
  )
BEGIN
  SELECT RAISE(ABORT, 'warunki zawartej umowy zmienia tylko aneks: warunki_od to chwila najnowszego aneksu, późniejsza niż poprzednie warunki');
END;

-- Wypowiedzenie albo odstąpienie późniejsze niż bieżące warunki kończy umowę bez aneksu: wpisuje
-- się koniec albo go skraca, ale nie na dzień przed dniem takiego zdarzenia. Pozostałe warunki
-- zmienia dalej tylko aneks (sprawa_umowy_warunki_zmienia_aneks).
CREATE TRIGGER sprawa_umowy_koniec_po_wypowiedzeniu_albo_odstapieniu
BEFORE UPDATE OF koniec ON sprawa_umowy_wpisy
WHEN OLD.stan IN ('zawarta', 'zakonczona')
  AND NEW.koniec IS NOT OLD.koniec
  AND NEW.okres_wypowiedzenia IS OLD.okres_wypowiedzenia
  AND NEW.wylacznosc IS OLD.wylacznosc
  AND NEW.wynagrodzenie_opis IS OLD.wynagrodzenie_opis
  AND NEW.wynagrodzenie_zdarzenie IS OLD.wynagrodzenie_zdarzenie
  AND NEW.warunki_od IS OLD.warunki_od
  AND EXISTS (
    SELECT 1
    FROM sprawa_umowy_zdarzenia
    WHERE umowa_id = NEW.id
      AND rodzaj IN ('wypowiedzenie', 'odstapienie')
      AND chwila > OLD.warunki_od
  )
  AND NOT coalesce(
    (OLD.koniec IS NULL OR NEW.koniec < OLD.koniec)
    AND EXISTS (
      SELECT 1
      FROM sprawa_umowy_zdarzenia
      WHERE umowa_id = NEW.id
        AND rodzaj IN ('wypowiedzenie', 'odstapienie')
        AND chwila > OLD.warunki_od
        AND date(chwila) <= NEW.koniec
    ),
    0
  )
BEGIN
  SELECT RAISE(ABORT, 'po wypowiedzeniu albo odstąpieniu koniec umowy wpisuje się albo skraca, nie na dzień przed dniem zdarzenia');
END;

-- Polisę zawartej umowy zmienia tylko zdarzenie nowa_polisa tej umowy, które wskazuje jej dokument.
CREATE TRIGGER sprawa_umowy_polise_zmienia_nowa_polisa
BEFORE UPDATE OF polisa_dokument_id ON sprawa_umowy_wpisy
WHEN OLD.stan IN ('zawarta', 'zakonczona')
  AND NEW.polisa_dokument_id IS NOT OLD.polisa_dokument_id
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_umowy_zdarzenia
    WHERE umowa_id = NEW.id
      AND rodzaj = 'nowa_polisa'
      AND dokument_id = NEW.polisa_dokument_id
  )
BEGIN
  SELECT RAISE(ABORT, 'polisę zawartej umowy zmienia tylko zdarzenie nowa_polisa, które wskazuje jej dokument');
END;

-- Dokument umowy i polisa zawartej umowy są podstawą umowy: nie zmieniają rodzaju ani tego, czego
-- dotyczą, i nie stają się odrzucone ani zbędne.
CREATE TRIGGER sprawa_umowy_dokument_zawartej_umowy_zostaje
BEFORE UPDATE ON sprawa_dokumenty_wpisy
WHEN (
    NEW.rodzaj IS NOT OLD.rodzaj
    OR NEW.dotyczy_uczestnik_id IS NOT OLD.dotyczy_uczestnik_id
    OR NEW.dotyczy_nieruchomosc_id IS NOT OLD.dotyczy_nieruchomosc_id
    OR (NEW.stan IN ('odrzucony', 'zbedny') AND NEW.stan IS NOT OLD.stan)
  )
  AND EXISTS (
    SELECT 1
    FROM sprawa_umowy_wpisy
    WHERE stan IN ('zawarta', 'zakonczona')
      AND OLD.id IN (dokument_id, polisa_dokument_id)
  )
BEGIN
  SELECT RAISE(ABORT, 'dokument, na którym stoi zawarta umowa, nie zmienia rodzaju ani tego, czego dotyczy, i nie jest odrzucany');
END;

-- Zdarzenia dotyczą umowy, która obowiązuje albo obowiązywała; reguły żądania i odstąpienia czytają
-- sposób zawarcia, który przed zawarciem jest jeszcze pusty. Projekt zakończony bez podpisu nie ma
-- chwili zawarcia, więc też nie ma zdarzeń.
CREATE TRIGGER sprawa_umowy_zdarzenie_przy_zawartej
BEFORE INSERT ON sprawa_umowy_zdarzenia
WHEN (
  SELECT stan NOT IN ('zawarta', 'zakonczona') OR zawarto IS NULL
  FROM sprawa_umowy_wpisy
  WHERE id = NEW.umowa_id
)
BEGIN
  SELECT RAISE(ABORT, 'zdarzenie powstaje tylko przy umowie zawartej albo zakończonej');
END;

-- Żądanie wskazuje zgodę zadanie_rozpoczecia_pracy złożoną przez stronę tej umowy.
CREATE TRIGGER sprawa_umowy_zadanie_wskazuje_zgode_strony
BEFORE INSERT ON sprawa_umowy_zdarzenia
WHEN NEW.rodzaj = 'zadanie_rozpoczecia_pracy'
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_zgody_wpisy AS zgoda
    JOIN sprawa_umowy_strony AS strona ON strona.uczestnik_id = zgoda.uczestnik_id
    WHERE zgoda.id = NEW.zgoda_id
      AND zgoda.rodzaj = 'zadanie_rozpoczecia_pracy'
      AND strona.umowa_id = NEW.umowa_id
  )
BEGIN
  SELECT RAISE(ABORT, 'żądanie rozpoczęcia pracy wskazuje zgodę zadanie_rozpoczecia_pracy strony umowy');
END;

-- Art. 15 ust. 3 upk: przy umowie poza lokalem (także w czasie nieumówionej wizyty) żądanie jest na
-- trwałym nośniku. Przy umowie na odległość przepis tego nie wymaga (art. 21 ust. 2 upk).
CREATE TRIGGER sprawa_umowy_zadanie_poza_lokalem_na_trwalym_nosniku
BEFORE INSERT ON sprawa_umowy_zdarzenia
WHEN NEW.rodzaj = 'zadanie_rozpoczecia_pracy'
  AND (
    SELECT sposob_zawarcia IN ('poza_lokalem', 'nieumowiona_wizyta')
    FROM sprawa_umowy_wpisy
    WHERE id = NEW.umowa_id
  )
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_zgody_wpisy
    WHERE id = NEW.zgoda_id
      AND trwaly_nosnik = 1
  )
BEGIN
  SELECT RAISE(ABORT, 'przy umowie poza lokalem żądanie rozpoczęcia pracy jest na trwałym nośniku');
END;

CREATE TRIGGER sprawa_umowy_odstapienie_wskazuje_oswiadczenie_konsumenta
BEFORE INSERT ON sprawa_umowy_zdarzenia
WHEN NEW.rodzaj = 'odstapienie'
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_zgody_wpisy AS zgoda
    JOIN sprawa_umowy_strony AS strona ON strona.uczestnik_id = zgoda.uczestnik_id
    JOIN sprawa_uczestnicy_wpisy AS uczestnik ON uczestnik.id = zgoda.uczestnik_id
    WHERE zgoda.id = NEW.zgoda_id
      AND zgoda.rodzaj = 'odstapienie'
      AND strona.umowa_id = NEW.umowa_id
      AND uczestnik.konsument = 1
  )
BEGIN
  SELECT RAISE(ABORT, 'odstąpienie wskazuje oświadczenie odstapienie konsumenta, który jest stroną umowy');
END;

CREATE TRIGGER sprawa_umowy_zdarzenia_bez_zmian
BEFORE UPDATE ON sprawa_umowy_zdarzenia
BEGIN
  SELECT RAISE(ABORT, 'zdarzenia umowy się nie zmienia; sprostowanie to nowe zdarzenie');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER sprawa_umowy_wpisy_bez_zastepowania
BEFORE INSERT ON sprawa_umowy_wpisy
WHEN EXISTS (
  SELECT 1
  FROM sprawa_umowy_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER sprawa_umowy_wpisy_bez_usuwania
BEFORE DELETE ON sprawa_umowy_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER sprawa_umowy_strony_bez_zastepowania
BEFORE INSERT ON sprawa_umowy_strony
WHEN EXISTS (
    SELECT 1
    FROM sprawa_umowy_strony
    WHERE id = NEW.id
  )
  OR EXISTS (
    SELECT 1
    FROM sprawa_umowy_strony
    WHERE umowa_id = NEW.umowa_id
      AND uczestnik_id = NEW.uczestnik_id
  )
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER sprawa_umowy_strony_bez_usuwania
BEFORE DELETE ON sprawa_umowy_strony
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
-- Bez podanego klucza NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- tabeli są dodatnie.
CREATE TRIGGER sprawa_umowy_zakres_bez_zastepowania
BEFORE INSERT ON sprawa_umowy_zakres
WHEN EXISTS (
    SELECT 1
    FROM sprawa_umowy_zakres
    WHERE id = NEW.id
  )
  OR EXISTS (
    SELECT 1
    FROM sprawa_umowy_zakres
    WHERE umowa_id = NEW.umowa_id
      AND czynnosc = NEW.czynnosc
  )
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER sprawa_umowy_zakres_bez_usuwania
BEFORE DELETE ON sprawa_umowy_zakres
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER sprawa_umowy_zdarzenia_bez_zastepowania
BEFORE INSERT ON sprawa_umowy_zdarzenia
WHEN EXISTS (
  SELECT 1
  FROM sprawa_umowy_zdarzenia
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER sprawa_umowy_zdarzenia_bez_usuwania
BEFORE DELETE ON sprawa_umowy_zdarzenia
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;
"#,
    ],
};

/// The values of column `wpisy.rodzaj`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyRodzaj {
    Posrednictwo,
    Zarzadzanie,
}

impl WpisyRodzaj {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyRodzaj] = &[
        WpisyRodzaj::Posrednictwo,
        WpisyRodzaj::Zarzadzanie,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyRodzaj::Posrednictwo => "posrednictwo",
            WpisyRodzaj::Zarzadzanie => "zarzadzanie",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyRodzaj {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "posrednictwo" => Ok(WpisyRodzaj::Posrednictwo),
                    "zarzadzanie" => Ok(WpisyRodzaj::Zarzadzanie),
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

/// The values of column `wpisy.stan`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyStan {
    Projekt,
    DoPodpisu,
    Zawarta,
    Zakonczona,
}

impl WpisyStan {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyStan] = &[
        WpisyStan::Projekt,
        WpisyStan::DoPodpisu,
        WpisyStan::Zawarta,
        WpisyStan::Zakonczona,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyStan::Projekt => "projekt",
            WpisyStan::DoPodpisu => "do_podpisu",
            WpisyStan::Zawarta => "zawarta",
            WpisyStan::Zakonczona => "zakonczona",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyStan {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "projekt" => Ok(WpisyStan::Projekt),
                    "do_podpisu" => Ok(WpisyStan::DoPodpisu),
                    "zawarta" => Ok(WpisyStan::Zawarta),
                    "zakonczona" => Ok(WpisyStan::Zakonczona),
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

/// The values of column `wpisy.forma`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisyForma {
    Pisemna,
    Elektroniczna,
}

impl WpisyForma {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisyForma] = &[WpisyForma::Pisemna, WpisyForma::Elektroniczna];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisyForma::Pisemna => "pisemna",
            WpisyForma::Elektroniczna => "elektroniczna",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisyForma {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "pisemna" => Ok(WpisyForma::Pisemna),
                    "elektroniczna" => Ok(WpisyForma::Elektroniczna),
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

/// The values of column `wpisy.sposob_zawarcia`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WpisySposobZawarcia {
    WLokaluBiura,
    PozaLokalem,
    NaOdleglosc,
    NieumowionaWizyta,
}

impl WpisySposobZawarcia {
    /// Every value, in the order of the catalog.
    pub const ALL: &[WpisySposobZawarcia] = &[
        WpisySposobZawarcia::WLokaluBiura,
        WpisySposobZawarcia::PozaLokalem,
        WpisySposobZawarcia::NaOdleglosc,
        WpisySposobZawarcia::NieumowionaWizyta,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            WpisySposobZawarcia::WLokaluBiura => "w_lokalu_biura",
            WpisySposobZawarcia::PozaLokalem => "poza_lokalem",
            WpisySposobZawarcia::NaOdleglosc => "na_odleglosc",
            WpisySposobZawarcia::NieumowionaWizyta => "nieumowiona_wizyta",
        }
    }
}

impl xmeta_room::FromSqlValue for WpisySposobZawarcia {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "w_lokalu_biura" => Ok(WpisySposobZawarcia::WLokaluBiura),
                    "poza_lokalem" => Ok(WpisySposobZawarcia::PozaLokalem),
                    "na_odleglosc" => Ok(WpisySposobZawarcia::NaOdleglosc),
                    "nieumowiona_wizyta" => Ok(WpisySposobZawarcia::NieumowionaWizyta),
                    other => Err(format!("{other} is no value of WpisySposobZawarcia")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for WpisySposobZawarcia {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// The values of column `zdarzenia.rodzaj`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ZdarzeniaRodzaj {
    Aneks,
    NowaPolisa,
    WezwanieOPolise,
    ZadanieRozpoczeciaPracy,
    Odstapienie,
    Wypowiedzenie,
    ZdarzenieWynagrodzenia,
}

impl ZdarzeniaRodzaj {
    /// Every value, in the order of the catalog.
    pub const ALL: &[ZdarzeniaRodzaj] = &[
        ZdarzeniaRodzaj::Aneks,
        ZdarzeniaRodzaj::NowaPolisa,
        ZdarzeniaRodzaj::WezwanieOPolise,
        ZdarzeniaRodzaj::ZadanieRozpoczeciaPracy,
        ZdarzeniaRodzaj::Odstapienie,
        ZdarzeniaRodzaj::Wypowiedzenie,
        ZdarzeniaRodzaj::ZdarzenieWynagrodzenia,
    ];
    /// The value as the database keeps it.
    pub fn as_str(self) -> &'static str {
        match self {
            ZdarzeniaRodzaj::Aneks => "aneks",
            ZdarzeniaRodzaj::NowaPolisa => "nowa_polisa",
            ZdarzeniaRodzaj::WezwanieOPolise => "wezwanie_o_polise",
            ZdarzeniaRodzaj::ZadanieRozpoczeciaPracy => "zadanie_rozpoczecia_pracy",
            ZdarzeniaRodzaj::Odstapienie => "odstapienie",
            ZdarzeniaRodzaj::Wypowiedzenie => "wypowiedzenie",
            ZdarzeniaRodzaj::ZdarzenieWynagrodzenia => "zdarzenie_wynagrodzenia",
        }
    }
}

impl xmeta_room::FromSqlValue for ZdarzeniaRodzaj {
    fn from_sql_value(value: &xmeta_room::SqlValue) -> Result<Self, String> {
        match value {
            xmeta_room::SqlValue::Text(text) => {
                match text.as_str() {
                    "aneks" => Ok(ZdarzeniaRodzaj::Aneks),
                    "nowa_polisa" => Ok(ZdarzeniaRodzaj::NowaPolisa),
                    "wezwanie_o_polise" => Ok(ZdarzeniaRodzaj::WezwanieOPolise),
                    "zadanie_rozpoczecia_pracy" => {
                        Ok(ZdarzeniaRodzaj::ZadanieRozpoczeciaPracy)
                    }
                    "odstapienie" => Ok(ZdarzeniaRodzaj::Odstapienie),
                    "wypowiedzenie" => Ok(ZdarzeniaRodzaj::Wypowiedzenie),
                    "zdarzenie_wynagrodzenia" => {
                        Ok(ZdarzeniaRodzaj::ZdarzenieWynagrodzenia)
                    }
                    other => Err(format!("{other} is no value of ZdarzeniaRodzaj")),
                }
            }
            other => Err(format!("expected text, got {other:?}")),
        }
    }
}

impl xmeta_room::ToSqlValue for ZdarzeniaRodzaj {
    fn to_sql_value(&self) -> xmeta_room::SqlValue {
        xmeta_room::SqlValue::Text(self.as_str().to_string())
    }
}

/// A row of `strony_umow`.
#[derive(Debug, Clone, PartialEq)]
pub struct StronyUmowRow {
    pub id: i64,
    pub umowa_id: String,
    pub uczestnik_id: String,
}

impl StronyUmowRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(StronyUmowRow {
            id: row.get("id")?,
            umowa_id: row.get("umowa_id")?,
            uczestnik_id: row.get("uczestnik_id")?,
        })
    }
}

/// Strony wszystkich umów teczki.
pub fn strony_umow(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<StronyUmowRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM sprawa_umowy_strony
ORDER BY umowa_id, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(StronyUmowRow::from_row).collect()
}

/// The parameters of `dopisz_strone_umowy`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DopiszStroneUmowyParams {
    pub umowa_id: String,
    pub uczestnik_id: String,
}

/// Klient, który jest stroną projektu umowy.
pub fn dopisz_strone_umowy(
    room: &impl xmeta_room::MetaRoom,
    params: &DopiszStroneUmowyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO sprawa_umowy_strony (umowa_id, uczestnik_id)
VALUES (:umowa_id, :uczestnik_id)"#,
                parameters: Vec::from([
                    ("umowa_id", xmeta_room::ToSqlValue::to_sql_value(&params.umowa_id)),
                    (
                        "uczestnik_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.uczestnik_id),
                    ),
                ]),
            },
        ],
    )
}

/// A row of `umowy`.
#[derive(Debug, Clone, PartialEq)]
pub struct UmowyRow {
    pub id: String,
    pub rodzaj: WpisyRodzaj,
    pub stan: WpisyStan,
    pub koniec_powod: Option<String>,
    pub forma: Option<WpisyForma>,
    pub sposob_zawarcia: Option<WpisySposobZawarcia>,
    pub zawarto: Option<String>,
    pub podpisal_za_biuro: Option<String>,
    pub dokument_id: Option<String>,
    pub polisa_dokument_id: Option<String>,
    pub nieruchomosc_id: Option<String>,
    pub poczatek: Option<String>,
    pub koniec: Option<String>,
    pub okres_wypowiedzenia: Option<String>,
    pub obie_strony: bool,
    pub wylacznosc: bool,
    pub wynagrodzenie_opis: Option<String>,
    pub wynagrodzenie_zdarzenie: Option<String>,
    pub warunki_od: Option<String>,
    pub informacja_o_odstapieniu_wysylka_id: Option<String>,
    pub odstapienie_do: Option<String>,
}

impl UmowyRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(UmowyRow {
            id: row.get("id")?,
            rodzaj: row.get("rodzaj")?,
            stan: row.get("stan")?,
            koniec_powod: row.get("koniec_powod")?,
            forma: row.get("forma")?,
            sposob_zawarcia: row.get("sposob_zawarcia")?,
            zawarto: row.get("zawarto")?,
            podpisal_za_biuro: row.get("podpisal_za_biuro")?,
            dokument_id: row.get("dokument_id")?,
            polisa_dokument_id: row.get("polisa_dokument_id")?,
            nieruchomosc_id: row.get("nieruchomosc_id")?,
            poczatek: row.get("poczatek")?,
            koniec: row.get("koniec")?,
            okres_wypowiedzenia: row.get("okres_wypowiedzenia")?,
            obie_strony: row.get("obie_strony")?,
            wylacznosc: row.get("wylacznosc")?,
            wynagrodzenie_opis: row.get("wynagrodzenie_opis")?,
            wynagrodzenie_zdarzenie: row.get("wynagrodzenie_zdarzenie")?,
            warunki_od: row.get("warunki_od")?,
            informacja_o_odstapieniu_wysylka_id: row
                .get("informacja_o_odstapieniu_wysylka_id")?,
            odstapienie_do: row.get("odstapienie_do")?,
        })
    }
}

/// Umowy klienta z biurem w kolejności wpisania.
pub fn umowy(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<UmowyRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM sprawa_umowy_wpisy
ORDER BY rowid"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(UmowyRow::from_row).collect()
}

/// A row of `praca_od_umow`.
#[derive(Debug, Clone, PartialEq)]
pub struct PracaOdUmowRow {
    pub umowa_id: String,
    pub od: String,
}

impl PracaOdUmowRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(PracaOdUmowRow {
            umowa_id: row.get("umowa_id")?,
            od: row.get("od")?,
        })
    }
}

/// Od kiedy biuro może pracować według każdej zawartej umowy (sprawa_umowy_praca_od).
pub fn praca_od_umow(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<PracaOdUmowRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT
  umowa_id,
  od
FROM sprawa_umowy_praca_od
ORDER BY umowa_id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(PracaOdUmowRow::from_row).collect()
}

/// The parameters of `przygotuj_umowe`.
#[derive(Debug, Clone, PartialEq)]
pub struct PrzygotujUmoweParams {
    pub id: String,
    pub rodzaj: WpisyRodzaj,
    pub nieruchomosc_id: Option<String>,
    pub poczatek: Option<String>,
    pub koniec: Option<String>,
    pub okres_wypowiedzenia: Option<String>,
    pub obie_strony: bool,
    pub wylacznosc: bool,
    pub wynagrodzenie_opis: Option<String>,
    pub wynagrodzenie_zdarzenie: Option<String>,
}

/// Projekt umowy pośrednictwa albo o zarządzanie; identyfikator (UUID) nadaje aplikacja, bo
/// wskazują go strony, zakres i kartoteka klientów.
pub fn przygotuj_umowe(
    room: &impl xmeta_room::MetaRoom,
    params: &PrzygotujUmoweParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO sprawa_umowy_wpisy (id, rodzaj, nieruchomosc_id, poczatek, koniec, okres_wypowiedzenia,
  obie_strony, wylacznosc, wynagrodzenie_opis, wynagrodzenie_zdarzenie, stan)
VALUES (:id, :rodzaj, :nieruchomosc_id, :poczatek, :koniec, :okres_wypowiedzenia, :obie_strony,
  :wylacznosc, :wynagrodzenie_opis, :wynagrodzenie_zdarzenie, 'projekt')"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("rodzaj", xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj)),
                    (
                        "nieruchomosc_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.nieruchomosc_id),
                    ),
                    ("poczatek", xmeta_room::ToSqlValue::to_sql_value(&params.poczatek)),
                    ("koniec", xmeta_room::ToSqlValue::to_sql_value(&params.koniec)),
                    (
                        "okres_wypowiedzenia",
                        xmeta_room::ToSqlValue::to_sql_value(&params.okres_wypowiedzenia),
                    ),
                    (
                        "obie_strony",
                        xmeta_room::ToSqlValue::to_sql_value(&params.obie_strony),
                    ),
                    (
                        "wylacznosc",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wylacznosc),
                    ),
                    (
                        "wynagrodzenie_opis",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wynagrodzenie_opis),
                    ),
                    (
                        "wynagrodzenie_zdarzenie",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.wynagrodzenie_zdarzenie,
                        ),
                    ),
                ]),
            },
        ],
    )
}

/// The parameters of `zmien_warunki_umowy`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZmienWarunkiUmowyParams {
    pub nieruchomosc_id: Option<String>,
    pub poczatek: Option<String>,
    pub koniec: Option<String>,
    pub okres_wypowiedzenia: Option<String>,
    pub obie_strony: bool,
    pub wylacznosc: bool,
    pub wynagrodzenie_opis: Option<String>,
    pub wynagrodzenie_zdarzenie: Option<String>,
    pub id: String,
}

/// Poprawka warunków projektu przed podpisem. Warunki zawartej umowy zmienia odnotuj_aneks_umowy.
pub fn zmien_warunki_umowy(
    room: &impl xmeta_room::MetaRoom,
    params: &ZmienWarunkiUmowyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE sprawa_umowy_wpisy
SET nieruchomosc_id = :nieruchomosc_id,
  poczatek = :poczatek,
  koniec = :koniec,
  okres_wypowiedzenia = :okres_wypowiedzenia,
  obie_strony = :obie_strony,
  wylacznosc = :wylacznosc,
  wynagrodzenie_opis = :wynagrodzenie_opis,
  wynagrodzenie_zdarzenie = :wynagrodzenie_zdarzenie
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "nieruchomosc_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.nieruchomosc_id),
                    ),
                    ("poczatek", xmeta_room::ToSqlValue::to_sql_value(&params.poczatek)),
                    ("koniec", xmeta_room::ToSqlValue::to_sql_value(&params.koniec)),
                    (
                        "okres_wypowiedzenia",
                        xmeta_room::ToSqlValue::to_sql_value(&params.okres_wypowiedzenia),
                    ),
                    (
                        "obie_strony",
                        xmeta_room::ToSqlValue::to_sql_value(&params.obie_strony),
                    ),
                    (
                        "wylacznosc",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wylacznosc),
                    ),
                    (
                        "wynagrodzenie_opis",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wynagrodzenie_opis),
                    ),
                    (
                        "wynagrodzenie_zdarzenie",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.wynagrodzenie_zdarzenie,
                        ),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `przekaz_umowe_do_podpisu`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PrzekazUmoweDoPodpisuParams {
    pub id: String,
}

/// Projekt gotowy do podpisu: strony sprawdzone, znacznik wątpliwości nie stoi.
pub fn przekaz_umowe_do_podpisu(
    room: &impl xmeta_room::MetaRoom,
    params: &PrzekazUmoweDoPodpisuParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE sprawa_umowy_wpisy
SET stan = 'do_podpisu'
WHERE id = :id"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `cofnij_umowe_do_projektu`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CofnijUmoweDoProjektuParams {
    pub id: String,
}

/// Umowa wraca z podpisu do poprawki.
pub fn cofnij_umowe_do_projektu(
    room: &impl xmeta_room::MetaRoom,
    params: &CofnijUmoweDoProjektuParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE sprawa_umowy_wpisy
SET stan = 'projekt'
WHERE id = :id"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_zawarcie_umowy`.
#[derive(Debug, Clone, PartialEq)]
pub struct OdnotujZawarcieUmowyParams {
    pub forma: WpisyForma,
    pub sposob_zawarcia: WpisySposobZawarcia,
    pub zawarto: String,
    pub podpisal_za_biuro: String,
    pub dokument_id: String,
    pub polisa_dokument_id: String,
    pub odstapienie_do: Option<String>,
    pub id: String,
}

/// Umowa podpisana: forma, sposób zawarcia, podpisany dokument i kopia polisy. Przy umowie
/// z konsumentem poza lokalem biura albo na odległość także ostatni dzień terminu na odstąpienie,
/// który liczy program.
pub fn odnotuj_zawarcie_umowy(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujZawarcieUmowyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE sprawa_umowy_wpisy
SET stan = 'zawarta',
  forma = :forma,
  sposob_zawarcia = :sposob_zawarcia,
  zawarto = :zawarto,
  podpisal_za_biuro = :podpisal_za_biuro,
  dokument_id = :dokument_id,
  polisa_dokument_id = :polisa_dokument_id,
  warunki_od = :zawarto,
  odstapienie_do = :odstapienie_do
WHERE id = :id"#,
                parameters: Vec::from([
                    ("forma", xmeta_room::ToSqlValue::to_sql_value(&params.forma)),
                    (
                        "sposob_zawarcia",
                        xmeta_room::ToSqlValue::to_sql_value(&params.sposob_zawarcia),
                    ),
                    ("zawarto", xmeta_room::ToSqlValue::to_sql_value(&params.zawarto)),
                    (
                        "podpisal_za_biuro",
                        xmeta_room::ToSqlValue::to_sql_value(&params.podpisal_za_biuro),
                    ),
                    (
                        "dokument_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dokument_id),
                    ),
                    (
                        "polisa_dokument_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.polisa_dokument_id),
                    ),
                    (
                        "odstapienie_do",
                        xmeta_room::ToSqlValue::to_sql_value(&params.odstapienie_do),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_aneks_umowy`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujAneksUmowyParams {
    pub koniec: Option<String>,
    pub okres_wypowiedzenia: Option<String>,
    pub wylacznosc: bool,
    pub wynagrodzenie_opis: Option<String>,
    pub wynagrodzenie_zdarzenie: Option<String>,
    pub warunki_od: String,
    pub id: String,
}

/// Nowe warunki zawartej umowy według aneksu wpisanego jako zdarzenie aneks; warunki_od to chwila
/// tego zdarzenia.
pub fn odnotuj_aneks_umowy(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujAneksUmowyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE sprawa_umowy_wpisy
SET koniec = :koniec,
  okres_wypowiedzenia = :okres_wypowiedzenia,
  wylacznosc = :wylacznosc,
  wynagrodzenie_opis = :wynagrodzenie_opis,
  wynagrodzenie_zdarzenie = :wynagrodzenie_zdarzenie,
  warunki_od = :warunki_od
WHERE id = :id"#,
                parameters: Vec::from([
                    ("koniec", xmeta_room::ToSqlValue::to_sql_value(&params.koniec)),
                    (
                        "okres_wypowiedzenia",
                        xmeta_room::ToSqlValue::to_sql_value(&params.okres_wypowiedzenia),
                    ),
                    (
                        "wylacznosc",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wylacznosc),
                    ),
                    (
                        "wynagrodzenie_opis",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wynagrodzenie_opis),
                    ),
                    (
                        "wynagrodzenie_zdarzenie",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.wynagrodzenie_zdarzenie,
                        ),
                    ),
                    (
                        "warunki_od",
                        xmeta_room::ToSqlValue::to_sql_value(&params.warunki_od),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_koniec_po_wypowiedzeniu`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujKoniecPoWypowiedzeniuParams {
    pub koniec: String,
    pub id: String,
}

/// Koniec zawartej umowy po zdarzeniu wypowiedzenie albo odstapienie późniejszym niż bieżące
/// warunki: wpisany albo krótszy niż dotąd, nie wcześniejszy niż dzień zdarzenia. Bez aneksu.
pub fn odnotuj_koniec_po_wypowiedzeniu(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujKoniecPoWypowiedzeniuParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE sprawa_umowy_wpisy
SET koniec = :koniec
WHERE id = :id"#,
                parameters: Vec::from([
                    ("koniec", xmeta_room::ToSqlValue::to_sql_value(&params.koniec)),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_nowa_polise_umowy`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujNowaPoliseUmowyParams {
    pub polisa_dokument_id: String,
    pub id: String,
}

/// Kopia nowej polisy pośrednika, którą wskazuje zdarzenie nowa_polisa tej umowy.
pub fn odnotuj_nowa_polise_umowy(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujNowaPoliseUmowyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE sprawa_umowy_wpisy
SET polisa_dokument_id = :polisa_dokument_id
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "polisa_dokument_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.polisa_dokument_id),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `odnotuj_informacje_o_odstapieniu`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OdnotujInformacjeOOdstapieniuParams {
    pub informacja_o_odstapieniu_wysylka_id: String,
    pub id: String,
}

/// Wysyłka, w której biuro poinformowało konsumenta o prawie odstąpienia; wpisuje się ją przed
/// zawarciem umowy (art. 12 upk), bo potem dane zawarcia się nie zmieniają.
pub fn odnotuj_informacje_o_odstapieniu(
    room: &impl xmeta_room::MetaRoom,
    params: &OdnotujInformacjeOOdstapieniuParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE sprawa_umowy_wpisy
SET informacja_o_odstapieniu_wysylka_id = :informacja_o_odstapieniu_wysylka_id
WHERE id = :id"#,
                parameters: Vec::from([
                    (
                        "informacja_o_odstapieniu_wysylka_id",
                        xmeta_room::ToSqlValue::to_sql_value(
                            &params.informacja_o_odstapieniu_wysylka_id,
                        ),
                    ),
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                ]),
            },
        ],
    )
}

/// The parameters of `zakoncz_umowe`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZakonczUmoweParams {
    pub koniec_powod: String,
    pub id: String,
}

/// Koniec umowy, np. wykonanie, wypowiedzenie, odstąpienie albo projekt bez podpisu.
pub fn zakoncz_umowe(
    room: &impl xmeta_room::MetaRoom,
    params: &ZakonczUmoweParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"UPDATE sprawa_umowy_wpisy
SET stan = 'zakonczona',
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

/// A row of `zakres_umow`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZakresUmowRow {
    pub id: i64,
    pub umowa_id: String,
    pub czynnosc: String,
}

impl ZakresUmowRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(ZakresUmowRow {
            id: row.get("id")?,
            umowa_id: row.get("umowa_id")?,
            czynnosc: row.get("czynnosc")?,
        })
    }
}

/// Czynności, na które pozwalają umowy teczki.
pub fn zakres_umow(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<ZakresUmowRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM sprawa_umowy_zakres
ORDER BY umowa_id, czynnosc"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(ZakresUmowRow::from_row).collect()
}

/// The parameters of `dopisz_czynnosc_umowy`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DopiszCzynnoscUmowyParams {
    pub umowa_id: String,
    pub czynnosc: String,
}

/// Czynność, na którą umowa pozwala biuru, nazwana jak w typie sprawy; tylko w projekcie umowy.
pub fn dopisz_czynnosc_umowy(
    room: &impl xmeta_room::MetaRoom,
    params: &DopiszCzynnoscUmowyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO sprawa_umowy_zakres (umowa_id, czynnosc)
VALUES (:umowa_id, :czynnosc)"#,
                parameters: Vec::from([
                    ("umowa_id", xmeta_room::ToSqlValue::to_sql_value(&params.umowa_id)),
                    ("czynnosc", xmeta_room::ToSqlValue::to_sql_value(&params.czynnosc)),
                ]),
            },
        ],
    )
}

/// A row of `zdarzenia_umow`.
#[derive(Debug, Clone, PartialEq)]
pub struct ZdarzeniaUmowRow {
    pub id: String,
    pub umowa_id: String,
    pub rodzaj: ZdarzeniaRodzaj,
    pub chwila: String,
    pub opis: Option<String>,
    pub dokument_id: Option<String>,
    pub wysylka_id: Option<String>,
    pub wplyw_id: Option<String>,
    pub zgoda_id: Option<String>,
}

impl ZdarzeniaUmowRow {
    /// The row from a row of the read, column by column.
    pub fn from_row(row: &xmeta_room::Row) -> Result<Self, xmeta_room::RoomError> {
        Ok(ZdarzeniaUmowRow {
            id: row.get("id")?,
            umowa_id: row.get("umowa_id")?,
            rodzaj: row.get("rodzaj")?,
            chwila: row.get("chwila")?,
            opis: row.get("opis")?,
            dokument_id: row.get("dokument_id")?,
            wysylka_id: row.get("wysylka_id")?,
            wplyw_id: row.get("wplyw_id")?,
            zgoda_id: row.get("zgoda_id")?,
        })
    }
}

/// Zdarzenia umów teczki w kolejności czasu.
pub fn zdarzenia_umow(
    room: &impl xmeta_room::MetaRoom,
) -> Result<Vec<ZdarzeniaUmowRow>, xmeta_room::RoomError> {
    let statement = xmeta_room::Statement {
        sql: r#"SELECT *
FROM sprawa_umowy_zdarzenia
ORDER BY chwila, id"#,
        parameters: Vec::new(),
    };
    let rows = room.read(&KIND, &statement)?;
    rows.iter().map(ZdarzeniaUmowRow::from_row).collect()
}

/// The parameters of `wpisz_zdarzenie_umowy`.
#[derive(Debug, Clone, PartialEq)]
pub struct WpiszZdarzenieUmowyParams {
    pub id: String,
    pub umowa_id: String,
    pub rodzaj: ZdarzeniaRodzaj,
    pub chwila: String,
    pub opis: Option<String>,
    pub dokument_id: Option<String>,
    pub wysylka_id: Option<String>,
    pub wplyw_id: Option<String>,
    pub zgoda_id: Option<String>,
}

/// Aneks, polisa, wezwanie, żądanie, odstąpienie, wypowiedzenie albo zdarzenie wynagrodzenia,
/// z dokumentem, wysyłką, wpływem albo zgodą, które je dowodzą; tylko przy zawartej albo
/// zakończonej umowie.
pub fn wpisz_zdarzenie_umowy(
    room: &impl xmeta_room::MetaRoom,
    params: &WpiszZdarzenieUmowyParams,
) -> Result<(), xmeta_room::RoomError> {
    room.write(
        &[&KIND],
        &[
            xmeta_room::Statement {
                sql: r#"INSERT INTO sprawa_umowy_zdarzenia (id, umowa_id, rodzaj, chwila, opis, dokument_id, wysylka_id,
  wplyw_id, zgoda_id)
VALUES (:id, :umowa_id, :rodzaj, :chwila, :opis, :dokument_id, :wysylka_id, :wplyw_id, :zgoda_id)"#,
                parameters: Vec::from([
                    ("id", xmeta_room::ToSqlValue::to_sql_value(&params.id)),
                    ("umowa_id", xmeta_room::ToSqlValue::to_sql_value(&params.umowa_id)),
                    ("rodzaj", xmeta_room::ToSqlValue::to_sql_value(&params.rodzaj)),
                    ("chwila", xmeta_room::ToSqlValue::to_sql_value(&params.chwila)),
                    ("opis", xmeta_room::ToSqlValue::to_sql_value(&params.opis)),
                    (
                        "dokument_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.dokument_id),
                    ),
                    (
                        "wysylka_id",
                        xmeta_room::ToSqlValue::to_sql_value(&params.wysylka_id),
                    ),
                    ("wplyw_id", xmeta_room::ToSqlValue::to_sql_value(&params.wplyw_id)),
                    ("zgoda_id", xmeta_room::ToSqlValue::to_sql_value(&params.zgoda_id)),
                ]),
            },
        ],
    )
}
