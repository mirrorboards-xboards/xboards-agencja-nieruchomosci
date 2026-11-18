-- Źródła: jak rzetelne okazały się osoby, instytucje, portale, rejestry, dokumenty i agenci, od
-- których agent coś wie. Po nieprawdzie źródła agent ocenia na nowo wszystko, co od niego wie
-- (operacja ustalenia_od_zrodla w stanowisko-ustalenia, a u kierownika działu także zwyczaje_od_zrodla).
--
-- Osobę z teczki, wykonawcę z rejestru wykonawców albo klienta z kartoteki wskazuje odnośnik do
-- jego wpisu, a nie jego dane: pamięć nie powtarza danych, które trzyma inny pokój (R1).
CREATE TABLE stanowisko_zrodla_wpisy (
  id TEXT PRIMARY KEY,
  nazwa TEXT,
  -- Rodzaj zgłoszenia (np. „pytanie o parking”) jest źródłem u Konsjerża, który ocenia zgłoszenia,
  -- a nie gości.
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('osoba', 'instytucja', 'firma', 'portal', 'kanal_sprzedazy', 'rejestr_publiczny', 'dokument', 'dane_rynkowe', 'rodzaj_zgloszenia', 'agent_zewnetrzny', 'agent_biura', 'czlowiek_biura')),
  -- Pokój i wiersz uczestnika teczki albo wpisu rejestru, tekstem.
  wpis_odnosnik TEXT,
  -- Tożsamość z XAuth/xkeys agenta albo człowieka biura.
  tozsamosc TEXT,
  ocena TEXT NOT NULL CHECK (ocena IN ('nieoceniona', 'rzetelne', 'zastrzezenia', 'nierzetelne')),
  oceniono TEXT,

  -- Wpis innego pokoju wskazuje odnośnik, więc jego nazwy pamięć nie powtarza.
  CONSTRAINT zrodlo_nazwane_albo_wskazane CHECK ((nazwa IS NULL) <> (wpis_odnosnik IS NULL)),
  CONSTRAINT ocena_z_data CHECK ((ocena = 'nieoceniona') = (oceniono IS NULL))
) STRICT;

-- Zdarzenia, na których stoi ocena źródła. To ślad, więc się ich nie zmienia; jedyną zmianą jest
-- odnotowanie, że zdarzenie weszło do raportu.
CREATE TABLE stanowisko_zrodla_zdarzenia (
  -- Klucz nadany przez bazę jest dodatni; zob. stanowisko_zrodla_zdarzenia_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  zrodlo_id TEXT NOT NULL REFERENCES stanowisko_zrodla_wpisy (id),
  zaszlo TEXT NOT NULL,
  -- Fakty (potwierdzenie, rozbieżność, nieprawda, spóźnienie, pytanie o drugą stronę), zakłócenia
  -- z profili stanowisk i rozstrzygnięcie zarzutu.
  rodzaj TEXT NOT NULL CHECK (rodzaj IN ('potwierdzenie', 'rozbieznosc', 'nieprawda', 'spoznienie', 'druga_strona', 'manipulacja', 'podszywanie', 'nacechowanie', 'dryf', 'przeciazenie', 'bledne_kolo', 'rozpoznanie', 'poroznienie', 'zarzut_potwierdzony', 'zarzut_odparty')),
  opis TEXT NOT NULL,
  -- Wiersz, który zdarzenie pokazuje, np. raport w teczce albo wpis dziennika wpływów: tabela
  -- i pokój z wierszem, tekstem.
  dotyczy_tabela TEXT,
  dotyczy_odnosnik TEXT,
  -- Kiedy agent podał zdarzenie w raporcie.
  zgloszono TEXT,

  CONSTRAINT dotyczy_tabela_razem_z_odnosnikiem CHECK ((dotyczy_tabela IS NULL) = (dotyczy_odnosnik IS NULL)),
  CONSTRAINT zgloszenie_nie_przed_zdarzeniem CHECK (zgloszono IS NULL OR zgloszono >= zaszlo)
) STRICT;

-- Obniżenie oceny sprawdza zdarzenia źródła, więc źródło nie może powstać od razu ocenione.
CREATE TRIGGER stanowisko_zrodla_powstaje_nieocenione
BEFORE INSERT ON stanowisko_zrodla_wpisy
WHEN NEW.ocena <> 'nieoceniona'
BEGIN
  SELECT RAISE(ABORT, 'źródło powstaje nieocenione; ocenę daje się po zdarzeniu');
END;

-- To, czym źródło jest, się nie zmienia; inne źródło to nowy wpis.
CREATE TRIGGER stanowisko_zrodla_tozsamosc_zrodla_niezmienna
BEFORE UPDATE ON stanowisko_zrodla_wpisy
WHEN NEW.nazwa IS NOT OLD.nazwa
  OR NEW.rodzaj IS NOT OLD.rodzaj
  OR NEW.wpis_odnosnik IS NOT OLD.wpis_odnosnik
  OR NEW.tozsamosc IS NOT OLD.tozsamosc
BEGIN
  SELECT RAISE(ABORT, 'nazwy, rodzaju ani wskazania źródła się nie zmienia; inne źródło to nowy wpis');
END;

-- Ocenę obniża tylko stwierdzony fakt. Sam zarzut (poróżnienie) jej nie obniża, dopóki agent go nie
-- potwierdzi, bo poróżnienie polega właśnie na zarzutach bez faktów. Pytanie o drugą stronę jest
-- faktem: naprawdę padło, a rozdział stron (R1) zabrania na nie odpowiadać.
CREATE TRIGGER stanowisko_zrodla_ocene_obniza_tylko_fakt
BEFORE UPDATE OF ocena ON stanowisko_zrodla_wpisy
WHEN NEW.ocena IN ('zastrzezenia', 'nierzetelne')
  AND NOT EXISTS (
    SELECT 1
    FROM stanowisko_zrodla_zdarzenia AS zdarzenie
    WHERE zdarzenie.zrodlo_id = NEW.id
      AND zdarzenie.rodzaj IN ('nieprawda', 'rozbieznosc', 'spoznienie', 'druga_strona', 'manipulacja', 'podszywanie', 'zarzut_potwierdzony')
  )
BEGIN
  SELECT RAISE(ABORT, 'ocenę źródła obniża tylko stwierdzony fakt, a nie sam zarzut');
END;

CREATE TRIGGER stanowisko_zrodla_zdarzenia_powstaje_niezgloszone
BEFORE INSERT ON stanowisko_zrodla_zdarzenia
WHEN NEW.zgloszono IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'zdarzenie źródła powstaje niezgłoszone; zgłoszenie odnotowuje się po raporcie');
END;

CREATE TRIGGER stanowisko_zrodla_zdarzenia_bez_zmian
BEFORE UPDATE ON stanowisko_zrodla_zdarzenia
WHEN OLD.zgloszono IS NOT NULL
  OR NEW.id IS NOT OLD.id
  OR NEW.zrodlo_id IS NOT OLD.zrodlo_id
  OR NEW.zaszlo IS NOT OLD.zaszlo
  OR NEW.rodzaj IS NOT OLD.rodzaj
  OR NEW.opis IS NOT OLD.opis
  OR NEW.dotyczy_tabela IS NOT OLD.dotyczy_tabela
  OR NEW.dotyczy_odnosnik IS NOT OLD.dotyczy_odnosnik
BEGIN
  SELECT RAISE(ABORT, 'zdarzenia źródła tylko się dopisuje; wolno raz odnotować zgłoszenie');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers. Bez podanego klucza
-- NEW.id tabeli z kluczem liczbowym jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze
-- są dodatnie.
CREATE TRIGGER stanowisko_zrodla_wpisy_bez_zastepowania
BEFORE INSERT ON stanowisko_zrodla_wpisy
WHEN EXISTS (
  SELECT 1
  FROM stanowisko_zrodla_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

CREATE TRIGGER stanowisko_zrodla_zdarzenia_bez_zastepowania
BEFORE INSERT ON stanowisko_zrodla_zdarzenia
WHEN EXISTS (
  SELECT 1
  FROM stanowisko_zrodla_zdarzenia
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- R9: pamięć trwa bez terminu; pomyłkę poprawia nowy wpis albo nowa ocena.
CREATE TRIGGER stanowisko_zrodla_wpisy_bez_usuwania
BEFORE DELETE ON stanowisko_zrodla_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;

CREATE TRIGGER stanowisko_zrodla_zdarzenia_bez_usuwania
BEFORE DELETE ON stanowisko_zrodla_zdarzenia
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;
