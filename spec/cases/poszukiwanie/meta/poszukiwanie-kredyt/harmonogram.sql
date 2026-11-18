-- name: harmonogram_kredytu :many
-- Kroki do wypłaty kredytu z datą przewidywaną i rzeczywistą.
SELECT *
FROM poszukiwanie_kredyt_harmonogram
ORDER BY przewidywana, id;

-- name: wpisz_krok_harmonogramu :exec
-- Krok drogi do wypłaty z datą przewidywaną, którą liczy program.
INSERT INTO poszukiwanie_kredyt_harmonogram (krok, przewidywana)
VALUES (:krok, :przewidywana);

-- name: przelicz_krok_harmonogramu :exec
-- Nowa data przewidywana z programu, dopóki krok się nie odbył.
UPDATE poszukiwanie_kredyt_harmonogram
SET przewidywana = :przewidywana
WHERE id = :id;

-- name: odnotuj_krok_harmonogramu :exec
-- Data, w której krok się odbył; potem kroku się nie zmienia.
-- param rzeczywista: string
UPDATE poszukiwanie_kredyt_harmonogram
SET rzeczywista = :rzeczywista
WHERE id = :id;
