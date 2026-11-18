-- name: uchwaly_wspolnoty :many
-- Uchwały wspólnoty albo spółdzielni o najmie krótkoterminowym w kolejności podjęcia.
SELECT *
FROM lokal_krotkoterminowy_lokal_uchwaly
ORDER BY data, id;

-- name: wpisz_uchwale_wspolnoty :exec
-- Uchwała z dniem podjęcia, treścią i dniem, w którym biuro się o niej dowiedziało.
INSERT INTO lokal_krotkoterminowy_lokal_uchwaly (data, tresc, dokument_id, dowiedziano)
VALUES (:data, :tresc, :dokument_id, :dowiedziano);
