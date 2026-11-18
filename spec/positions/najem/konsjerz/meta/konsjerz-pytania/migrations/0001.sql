-- Pytania gości: o co goście tego lokalu pytają i jaka odpowiedź im pomogła, np. że wjazd do garażu
-- jest od ul. Bocznej. Z tych wpisów Konsjerż proponuje Zarządcy zmiany w opisie lokalu; propozycja
-- idzie notatką w teczce lokalu, a wprowadza ją Zarządca.
--
-- Żadna kolumna nie wskazuje gościa ani rozmowy: pamięć Konsjerża nie może się stać kartoteką
-- gości, a rozmowy leżą w kanale i w teczce. Lepsza odpowiedź na to samo pytanie to nowy wiersz,
-- który zastępuje poprzedni, więc wystąpienia zostają przy odpowiedzi, której dotyczyły.
CREATE TABLE konsjerz_pytania_wpisy (
  -- Klucz nadany przez bazę jest dodatni; zob. konsjerz_pytania_wpisy_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  -- Np. „wjazd do garażu”.
  temat TEXT NOT NULL CHECK (temat <> ''),
  odpowiedz TEXT NOT NULL CHECK (odpowiedz <> ''),
  zapisano TEXT NOT NULL,
  -- Kiedy i którą notatką w teczce lokalu Konsjerż zaproponował Zarządcy zmianę opisu lokalu.
  propozycja_przekazano TEXT,
  notatka_odnosnik TEXT,
  zastapione_przez_id INTEGER REFERENCES konsjerz_pytania_wpisy (id),

  CONSTRAINT propozycja_z_notatka CHECK ((propozycja_przekazano IS NULL) = (notatka_odnosnik IS NULL)),
  CONSTRAINT propozycja_nie_przed_zapisem CHECK (propozycja_przekazano IS NULL OR propozycja_przekazano >= zapisano),
  CONSTRAINT zastapione_nie_przez_siebie CHECK (zastapione_przez_id IS NULL OR zastapione_przez_id <> id)
) STRICT;

-- Każde zadanie pytania i to, czy odpowiedź pomogła. Wystąpienie nie wskazuje gościa.
CREATE TABLE konsjerz_pytania_wystapienia (
  -- Klucz nadany przez bazę jest dodatni; zob. konsjerz_pytania_wystapienia_bez_zastepowania.
  id INTEGER PRIMARY KEY CHECK (id > 0),
  pytanie_id INTEGER NOT NULL REFERENCES konsjerz_pytania_wpisy (id),
  zadano TEXT NOT NULL,
  -- 1, gdy odpowiedź pomogła, 0, gdy nie; NULL, dopóki Konsjerż tego nie wie.
  pomogla INTEGER CHECK (pomogla IN (0, 1))
) STRICT;

-- Propozycja i zastąpienie przychodzą później, więc pytanie powstaje aktualne i bez propozycji.
CREATE TRIGGER konsjerz_pytania_powstaje_aktualne
BEFORE INSERT ON konsjerz_pytania_wpisy
WHEN NEW.propozycja_przekazano IS NOT NULL
  OR NEW.zastapione_przez_id IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'pytanie powstaje aktualne i bez propozycji; obie rzeczy odnotowuje się później');
END;

-- Pytanie z odpowiedzią się nie zmienia; wolno raz odnotować propozycję i raz zastąpienie.
CREATE TRIGGER konsjerz_pytania_bez_zmian
BEFORE UPDATE ON konsjerz_pytania_wpisy
WHEN NEW.id IS NOT OLD.id
  OR NEW.temat IS NOT OLD.temat
  OR NEW.odpowiedz IS NOT OLD.odpowiedz
  OR NEW.zapisano IS NOT OLD.zapisano
  OR (OLD.propozycja_przekazano IS NOT NULL AND (
    NEW.propozycja_przekazano IS NOT OLD.propozycja_przekazano
    OR NEW.notatka_odnosnik IS NOT OLD.notatka_odnosnik
  ))
  OR (OLD.zastapione_przez_id IS NOT NULL AND NEW.zastapione_przez_id IS NOT OLD.zastapione_przez_id)
BEGIN
  SELECT RAISE(ABORT, 'pytania ani odpowiedzi się nie zmienia; lepsza odpowiedź to nowy wiersz');
END;

-- Nowa odpowiedź zastępuje odpowiedź na ten sam temat i nie jest starsza od niej.
CREATE TRIGGER konsjerz_pytania_zastepuje_ten_sam_temat
BEFORE UPDATE OF zastapione_przez_id ON konsjerz_pytania_wpisy
WHEN NEW.zastapione_przez_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM konsjerz_pytania_wpisy AS nowe
    WHERE nowe.id = NEW.zastapione_przez_id
      AND nowe.temat = OLD.temat
      AND nowe.zapisano >= OLD.zapisano
  )
BEGIN
  SELECT RAISE(ABORT, 'pytanie zastępuje nowsza odpowiedź na ten sam temat');
END;

-- Wystąpienie dotyczy aktualnej odpowiedzi i nie poprzedza jej zapisania.
CREATE TRIGGER konsjerz_pytania_wystapienie_aktualnej_odpowiedzi
BEFORE INSERT ON konsjerz_pytania_wystapienia
WHEN NOT EXISTS (
  SELECT 1
  FROM konsjerz_pytania_wpisy AS pytanie
  WHERE pytanie.id = NEW.pytanie_id
    AND pytanie.zastapione_przez_id IS NULL
    AND pytanie.zapisano <= NEW.zadano
)
BEGIN
  SELECT RAISE(ABORT, 'wystąpienie dotyczy aktualnej odpowiedzi i nie poprzedza jej zapisania');
END;

CREATE TRIGGER konsjerz_pytania_wystapienie_powstaje_bez_oceny
BEFORE INSERT ON konsjerz_pytania_wystapienia
WHEN NEW.pomogla IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'wystąpienie powstaje bez oceny; to, czy odpowiedź pomogła, odnotowuje się później');
END;

CREATE TRIGGER konsjerz_pytania_wystapienia_bez_zmian
BEFORE UPDATE ON konsjerz_pytania_wystapienia
WHEN OLD.pomogla IS NOT NULL
  OR NEW.id IS NOT OLD.id
  OR NEW.pytanie_id IS NOT OLD.pytanie_id
  OR NEW.zadano IS NOT OLD.zadano
BEGIN
  SELECT RAISE(ABORT, 'wystąpienia tylko się dopisuje; wolno raz odnotować, czy odpowiedź pomogła');
END;

-- Bez zastępowania i usuwania --------------------------------------------------------------------

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers. Bez podanego klucza
-- NEW.id jest tu -1 albo NULL i nie wskazuje żadnego wiersza, bo klucze są dodatnie.
CREATE TRIGGER konsjerz_pytania_wpisy_bez_zastepowania
BEFORE INSERT ON konsjerz_pytania_wpisy
WHEN EXISTS (
  SELECT 1
  FROM konsjerz_pytania_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

CREATE TRIGGER konsjerz_pytania_wystapienia_bez_zastepowania
BEFORE INSERT ON konsjerz_pytania_wystapienia
WHEN EXISTS (
  SELECT 1
  FROM konsjerz_pytania_wystapienia
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pamięci nie zastępuje się nowym wpisem (INSERT OR REPLACE, upsert); zmiana idzie przez UPDATE');
END;

-- R9: pamięć trwa bez terminu.
CREATE TRIGGER konsjerz_pytania_wpisy_bez_usuwania
BEFORE DELETE ON konsjerz_pytania_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;

CREATE TRIGGER konsjerz_pytania_wystapienia_bez_usuwania
BEFORE DELETE ON konsjerz_pytania_wystapienia
BEGIN
  SELECT RAISE(ABORT, 'wierszy pamięci się nie usuwa (R9); pomyłkę poprawia nowy wpis');
END;
