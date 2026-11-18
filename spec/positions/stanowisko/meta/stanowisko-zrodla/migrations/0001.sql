-- Źródła: jak rzetelne okazały się osoby, instytucje, rejestry i agenci, od których agent coś wie.
-- Po nieprawdzie źródła agent ocenia na nowo wszystko, co od niego wie (operacja
-- ustalenia_od_zrodla w stanowisko-ustalenia).
--
-- Osobę ze sprawy wskazuje odnośnik do uczestnika w teczce albo do kartoteki, a nie jej dane.
CREATE TABLE stanowisko_zrodla_wpisy (
  id TEXT PRIMARY KEY,
  nazwa TEXT,
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('osoba', 'instytucja', 'agent_zewnetrzny', 'agent_kancelarii', 'czlowiek_kancelarii', 'rejestr_publiczny')),
  -- Pokój i wiersz uczestnika, tekstem.
  uczestnik_odnosnik TEXT,
  -- Tożsamość z XAuth/xkeys agenta albo człowieka kancelarii.
  tozsamosc TEXT,
  ocena TEXT NOT NULL CHECK (ocena IN ('nieoceniona', 'rzetelne', 'zastrzezenia', 'nierzetelne')),
  oceniono TEXT,

  -- Uczestnika sprawy wskazuje odnośnik, więc jego nazwy pamięć nie powtarza.
  CONSTRAINT zrodlo_nazwane_albo_wskazane CHECK ((nazwa IS NULL) <> (uczestnik_odnosnik IS NULL)),
  CONSTRAINT ocena_z_data CHECK ((ocena = 'nieoceniona') = (oceniono IS NULL))
) STRICT;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- triggerów UPDATE i DELETE, bo pokój nie włącza recursive_triggers; zmianę wiersza robi UPDATE.
CREATE TRIGGER stanowisko_zrodla_wpisy_bez_zastepowania
BEFORE INSERT ON stanowisko_zrodla_wpisy
WHEN EXISTS (
  SELECT 1
  FROM stanowisko_zrodla_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- Zdarzenia, na których stoi ocena źródła. To ślad, więc się ich nie zmienia ani nie usuwa; jedyną
-- zmianą jest odnotowanie, że zdarzenie weszło do raportu.
CREATE TABLE stanowisko_zrodla_zdarzenia (
  -- Klucz nadany przez bazę jest dodatni; zob. stanowisko_zrodla_zdarzenia_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  zrodlo_id TEXT NOT NULL REFERENCES stanowisko_zrodla_wpisy (id),
  zaszlo TEXT NOT NULL,
  -- Fakty (potwierdzenie, rozbieżność, nieprawda, spóźnienie), zakłócenia z profili stanowisk
  -- i rozstrzygnięcie zarzutu.
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('potwierdzenie', 'rozbieznosc', 'nieprawda', 'spoznienie', 'manipulacja', 'podszywanie', 'nacechowanie', 'dryf', 'przeciazenie', 'bledne_kolo', 'rozpoznanie', 'poroznienie', 'zarzut_potwierdzony', 'zarzut_odparty')),
  opis TEXT NOT NULL,
  -- Wiersz teczki, który zdarzenie pokazuje: tabela i pokój z wierszem, tekstem.
  dotyczy_tabela TEXT,
  dotyczy_odnosnik TEXT,
  -- Kiedy agent podał zdarzenie w raporcie.
  zgloszono TEXT,

  CONSTRAINT dotyczy_tabela_razem_z_odnosnikiem CHECK ((dotyczy_tabela IS NULL) = (dotyczy_odnosnik IS NULL)),
  CONSTRAINT zgloszenie_nie_przed_zdarzeniem CHECK (zgloszono IS NULL OR zgloszono >= zaszlo)
) STRICT;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- triggerów UPDATE i DELETE, bo pokój nie włącza recursive_triggers; zmianę wiersza robi UPDATE.
CREATE TRIGGER stanowisko_zrodla_zdarzenia_bez_zastepowania
BEFORE INSERT ON stanowisko_zrodla_zdarzenia
WHEN EXISTS (
  SELECT 1
  FROM stanowisko_zrodla_zdarzenia
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- Obniżenie oceny sprawdza zdarzenia źródła, więc źródło nie może powstać od razu ocenione.
CREATE TRIGGER stanowisko_zrodla_powstaje_nieocenione
BEFORE INSERT ON stanowisko_zrodla_wpisy
WHEN NEW.ocena <> 'nieoceniona'
BEGIN
  SELECT RAISE(ABORT, 'źródło powstaje nieocenione; ocenę daje się po zdarzeniu');
END;

-- Ocenę obniża tylko stwierdzony fakt. Sam zarzut (poróżnienie) jej nie obniża, dopóki agent go nie
-- potwierdzi, bo poróżnienie polega właśnie na zarzutach bez faktów.
CREATE TRIGGER stanowisko_zrodla_ocene_obniza_tylko_fakt
BEFORE UPDATE OF ocena ON stanowisko_zrodla_wpisy
WHEN NEW.ocena IN ('zastrzezenia', 'nierzetelne')
  AND NOT EXISTS (
    SELECT 1
    FROM stanowisko_zrodla_zdarzenia AS zdarzenie
    WHERE zdarzenie.zrodlo_id = NEW.id
      AND zdarzenie.rodzaj IN ('nieprawda', 'rozbieznosc', 'spoznienie', 'manipulacja', 'podszywanie', 'zarzut_potwierdzony')
  )
BEGIN
  SELECT RAISE(ABORT, 'ocenę źródła obniża tylko stwierdzony fakt, a nie sam zarzut');
END;

CREATE TRIGGER stanowisko_zrodla_zdarzenia_bez_zmian
BEFORE UPDATE ON stanowisko_zrodla_zdarzenia
WHEN OLD.zgloszono IS NOT NULL
  OR NEW.zrodlo_id IS NOT OLD.zrodlo_id
  OR NEW.zaszlo IS NOT OLD.zaszlo
  OR NEW.rodzaj IS NOT OLD.rodzaj
  OR NEW.opis IS NOT OLD.opis
  OR NEW.dotyczy_tabela IS NOT OLD.dotyczy_tabela
  OR NEW.dotyczy_odnosnik IS NOT OLD.dotyczy_odnosnik
BEGIN
  SELECT RAISE(ABORT, 'zdarzenia źródła tylko się dopisuje; wolno raz odnotować zgłoszenie');
END;

CREATE TRIGGER stanowisko_zrodla_zdarzenia_bez_usuwania
BEFORE DELETE ON stanowisko_zrodla_zdarzenia
BEGIN
  SELECT RAISE(ABORT, 'zdarzenia źródła tylko się dopisuje; usuwanie nie jest dozwolone');
END;
