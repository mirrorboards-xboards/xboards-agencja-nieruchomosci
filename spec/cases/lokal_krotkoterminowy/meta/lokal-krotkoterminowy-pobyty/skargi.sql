-- name: skargi_sasiadow :many
-- Skargi sąsiadów i wspólnoty w kolejności czasu.
SELECT *
FROM lokal_krotkoterminowy_pobyty_skargi
ORDER BY chwila, id;

-- name: wpisz_skarge :exec
-- Skarga: kto, czego dotyczy i którego pobytu, jeśli wiadomo.
INSERT INTO lokal_krotkoterminowy_pobyty_skargi (chwila, kto, tresc, pobyt_id)
VALUES (:chwila, :kto, :tresc, :pobyt_id);

-- name: wskaz_odpowiedz_na_skarge :exec
-- Projekt odpowiedzi na skargę, zanim wyjdzie; czeka on na zgodę człowieka.
-- param odpowiedz_wysylka_id: string
UPDATE lokal_krotkoterminowy_pobyty_skargi
SET odpowiedz_wysylka_id = :odpowiedz_wysylka_id
WHERE id = :id;
