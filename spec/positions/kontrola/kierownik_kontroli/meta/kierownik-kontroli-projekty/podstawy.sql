-- name: podstawy_projektu :many
-- Wpisy w rejestrze kontroli, na których stoi projekt.
SELECT *
FROM kierownik_kontroli_projekty_podstawy
WHERE projekt_id = :projekt_id
ORDER BY id;

-- name: dopisz_podstawe :exec
-- Wpis w rejestrze kontroli, na którym stoi projekt w toku.
INSERT INTO kierownik_kontroli_projekty_podstawy (projekt_id, rejestr_odnosnik)
VALUES (:projekt_id, :rejestr_odnosnik);
