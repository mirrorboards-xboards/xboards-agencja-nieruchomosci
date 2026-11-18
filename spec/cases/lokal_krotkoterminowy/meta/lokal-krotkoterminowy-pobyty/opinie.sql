-- name: opinie_gosci :many
-- Opinie gości w kolejności publikacji.
SELECT *
FROM lokal_krotkoterminowy_pobyty_opinie
ORDER BY opublikowano, id;

-- name: wpisz_opinie_goscia :exec
-- Opinia gościa z oceną 1–5 i dniem publikacji.
INSERT INTO lokal_krotkoterminowy_pobyty_opinie (id, pobyt_id, ocena, opublikowano)
VALUES (:id, :pobyt_id, :ocena, :opublikowano);

-- name: wskaz_odpowiedz_na_opinie :exec
-- Projekt odpowiedzi na opinię, zanim wyjdzie; przy ocenie 3 i niższej czeka on na zgodę.
-- param odpowiedz_wysylka_id: string
UPDATE lokal_krotkoterminowy_pobyty_opinie
SET odpowiedz_wysylka_id = :odpowiedz_wysylka_id
WHERE id = :id;
