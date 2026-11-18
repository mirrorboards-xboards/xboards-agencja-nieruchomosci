-- name: projekty_ocen :many
-- Projekty oceny ryzyka biura, najnowsze pierwsze.
SELECT *
FROM kierownik_kontroli_oceny_wpisy
ORDER BY rozpoczeto DESC, id;

-- name: rozpocznij_projekt_oceny :exec
-- Nowy projekt oceny ryzyka biura, np. przy corocznym przeglądzie.
INSERT INTO kierownik_kontroli_oceny_wpisy (rozpoczeto)
VALUES (:rozpoczeto);

-- name: odnotuj_przeglad_oceny :exec
-- Kierownik przejrzał czynniki projektu.
-- param przejrzano: string
UPDATE kierownik_kontroli_oceny_wpisy
SET przejrzano = :przejrzano
WHERE id = :id;

-- name: przekaz_ocene_do_zatwierdzenia :exec
-- Projekt trafił do właściciela biura do zatwierdzenia.
-- param przekazano_do_zatwierdzenia: string
UPDATE kierownik_kontroli_oceny_wpisy
SET przekazano_do_zatwierdzenia = :przekazano_do_zatwierdzenia
WHERE id = :id;

-- name: odnotuj_zatwierdzenie_oceny :exec
-- Zatwierdzona ocena jest w rejestrze kontroli; projekt odtąd tylko ją wskazuje.
-- param rejestr_odnosnik: string
UPDATE kierownik_kontroli_oceny_wpisy
SET rejestr_odnosnik = :rejestr_odnosnik
WHERE id = :id;
