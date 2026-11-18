-- Obszary lokali krótkoterminowych: granice każdego obszaru, zatwierdzone przez człowieka, jego
-- Koordynator operacji i lokale, które do niego należą.
--
-- Granice proponuje kierownik, a zatwierdza człowiek podpisem w wiadomości; kierownik przenosi
-- podpis do wiersza, a sprawdza go aplikacja. Zatwierdzonych granic się nie zmienia: inne granice
-- to nowy obszar, a stary się znosi. Koordynatora operacji kierownik powołuje w kierownik-dzialu-agenci,
-- a obszar wskazuje jego wiersz, bo Koordynator nie ma teczki.
CREATE TABLE kierownik_najmu_obszary_wpisy (
  -- Klucz nadany przez bazę jest dodatni; zob. kierownik_najmu_obszary_wpisy_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  nazwa TEXT NOT NULL CHECK (nazwa <> ''),
  -- Np. „Wrocław, Stare Miasto w granicach ul. Kazimierza Wielkiego i fosy”.
  granice TEXT NOT NULL CHECK (granice <> ''),
  zaproponowano TEXT NOT NULL,
  zatwierdzil TEXT,
  podpis TEXT,
  zatwierdzono TEXT,
  koordynator_id TEXT REFERENCES kierownik_dzialu_agenci_wpisy (id),
  zniesiono TEXT,

  CONSTRAINT zatwierdzenie_w_komplecie CHECK (
    (zatwierdzil IS NULL) = (podpis IS NULL)
    AND (zatwierdzil IS NULL) = (zatwierdzono IS NULL)
  ),
  CONSTRAINT zatwierdzenie_nie_przed_propozycja CHECK (zatwierdzono IS NULL OR zatwierdzono >= zaproponowano),
  CONSTRAINT koordynator_obszaru_zatwierdzonego CHECK (koordynator_id IS NULL OR zatwierdzil IS NOT NULL),
  CONSTRAINT zniesienie_nie_przed_propozycja CHECK (zniesiono IS NULL OR zniesiono >= zaproponowano)
) STRICT;

-- Lokal (jego teczka) należy do obszaru od dnia do dnia; naraz do jednego obszaru.
CREATE TABLE kierownik_najmu_obszary_lokale (
  -- Klucz nadany przez bazę jest dodatni; zob. kierownik_najmu_obszary_lokale_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  obszar_id INTEGER NOT NULL REFERENCES kierownik_najmu_obszary_wpisy (id),
  -- Pokój teczki lokalu krótkoterminowego, tekstem.
  teczka_odnosnik TEXT NOT NULL CHECK (teczka_odnosnik <> ''),
  od TEXT NOT NULL,
  do_dnia TEXT,

  CONSTRAINT koniec_nie_przed_poczatkiem CHECK (do_dnia IS NULL OR do_dnia >= od)
) STRICT;

CREATE UNIQUE INDEX kierownik_najmu_obszary_lokal_w_jednym_obszarze
ON kierownik_najmu_obszary_lokale (teczka_odnosnik)
WHERE do_dnia IS NULL;

-- Obszary ----------------------------------------------------------------------------------------

CREATE TRIGGER kierownik_najmu_obszary_powstaje_jako_propozycja
BEFORE INSERT ON kierownik_najmu_obszary_wpisy
WHEN NEW.zatwierdzil IS NOT NULL
  OR NEW.koordynator_id IS NOT NULL
  OR NEW.zniesiono IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'obszar powstaje jako propozycja kierownika; granice zatwierdza człowiek');
END;

-- Koordynatorem obszaru jest agent na stanowisku Koordynatora operacji, który nie został wymieniony.
CREATE TRIGGER kierownik_najmu_obszary_koordynatorem_koordynator_operacji
BEFORE UPDATE OF koordynator_id ON kierownik_najmu_obszary_wpisy
WHEN NEW.koordynator_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM kierownik_dzialu_agenci_wpisy AS agent
    WHERE agent.id = NEW.koordynator_id
      AND agent.stanowisko = 'koordynator_operacji'
      AND agent.stan <> 'wymieniony'
  )
BEGIN
  SELECT RAISE(ABORT, 'koordynatorem obszaru jest niewymieniony agent na stanowisku Koordynatora operacji');
END;

-- Podpis obejmuje granice, więc zatwierdzonego obszaru nie zmienia się poza Koordynatorem i raz
-- odnotowanym zniesieniem; zniesionego nie zmienia się wcale.
CREATE TRIGGER kierownik_najmu_obszary_bez_zmian
BEFORE UPDATE ON kierownik_najmu_obszary_wpisy
WHEN OLD.zniesiono IS NOT NULL
  OR NEW.id IS NOT OLD.id
  OR NEW.zaproponowano IS NOT OLD.zaproponowano
  OR (OLD.zatwierdzil IS NOT NULL AND (
    NEW.nazwa IS NOT OLD.nazwa
    OR NEW.granice IS NOT OLD.granice
    OR NEW.zatwierdzil IS NOT OLD.zatwierdzil
    OR NEW.podpis IS NOT OLD.podpis
    OR NEW.zatwierdzono IS NOT OLD.zatwierdzono
  ))
BEGIN
  SELECT RAISE(ABORT, 'zatwierdzonych granic obszaru się nie zmienia, a zniesionego obszaru wcale; inne granice to nowy obszar');
END;

-- Obszar znosi się dopiero, gdy nie ma już w nim lokali.
CREATE TRIGGER kierownik_najmu_obszary_zniesiony_bez_lokali
BEFORE UPDATE OF zniesiono ON kierownik_najmu_obszary_wpisy
WHEN NEW.zniesiono IS NOT NULL
  AND EXISTS (
    SELECT 1
    FROM kierownik_najmu_obszary_lokale AS lokal
    WHERE lokal.obszar_id = NEW.id
      AND lokal.do_dnia IS NULL
  )
BEGIN
  SELECT RAISE(ABORT, 'obszar znosi się dopiero, gdy nie ma w nim lokali');
END;

-- Lokale -----------------------------------------------------------------------------------------

CREATE TRIGGER kierownik_najmu_obszary_lokal_powstaje_w_obszarze
BEFORE INSERT ON kierownik_najmu_obszary_lokale
WHEN NEW.do_dnia IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'lokal powstaje w obszarze; koniec przynależności odnotowuje się później');
END;

CREATE TRIGGER kierownik_najmu_obszary_lokal_do_zatwierdzonego_obszaru
BEFORE INSERT ON kierownik_najmu_obszary_lokale
WHEN NOT EXISTS (
  SELECT 1
  FROM kierownik_najmu_obszary_wpisy AS obszar
  WHERE obszar.id = NEW.obszar_id
    AND obszar.zatwierdzil IS NOT NULL
    AND obszar.zniesiono IS NULL
)
BEGIN
  SELECT RAISE(ABORT, 'lokal należy tylko do obszaru zatwierdzonego i niezniesionego');
END;

-- Drugie przypisanie lokalu z innym kluczem zderzyłoby się z indeksem, a INSERT OR REPLACE usunąłby
-- wtedy pierwsze z ominięciem blokady usuwania; ten trigger odmawia wcześniej.
CREATE TRIGGER kierownik_najmu_obszary_lokal_w_jednym_obszarze_naraz
BEFORE INSERT ON kierownik_najmu_obszary_lokale
WHEN EXISTS (
  SELECT 1
  FROM kierownik_najmu_obszary_lokale
  WHERE teczka_odnosnik = NEW.teczka_odnosnik
    AND do_dnia IS NULL
    AND id IS NOT NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'lokal należy naraz do jednego obszaru; nowy obszar przychodzi po końcu starego');
END;

CREATE TRIGGER kierownik_najmu_obszary_lokale_bez_zmian
BEFORE UPDATE ON kierownik_najmu_obszary_lokale
WHEN OLD.do_dnia IS NOT NULL
  OR NEW.id IS NOT OLD.id
  OR NEW.obszar_id IS NOT OLD.obszar_id
  OR NEW.teczka_odnosnik IS NOT OLD.teczka_odnosnik
  OR NEW.od IS NOT OLD.od
BEGIN
  SELECT RAISE(ABORT, 'przynależności lokalu do obszaru się nie zmienia; wolno raz odnotować jej koniec');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers. Bez podanego klucza
-- NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze są dodatnie.
CREATE TRIGGER kierownik_najmu_obszary_wpisy_bez_zastepowania
BEFORE INSERT ON kierownik_najmu_obszary_wpisy
WHEN EXISTS (
  SELECT 1
  FROM kierownik_najmu_obszary_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

CREATE TRIGGER kierownik_najmu_obszary_lokale_bez_zastepowania
BEFORE INSERT ON kierownik_najmu_obszary_lokale
WHEN EXISTS (
  SELECT 1
  FROM kierownik_najmu_obszary_lokale
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- R9: zniesiony obszar i dawna przynależność lokalu zostają jako historia.
CREATE TRIGGER kierownik_najmu_obszary_wpisy_bez_usuwania
BEFORE DELETE ON kierownik_najmu_obszary_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;

CREATE TRIGGER kierownik_najmu_obszary_lokale_bez_usuwania
BEFORE DELETE ON kierownik_najmu_obszary_lokale
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;
