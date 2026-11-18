-- name: przeglady_lokalu :many
-- Przeglądy lokalu w kolejności dat.
SELECT *
FROM lokal_krotkoterminowy_lokal_przeglady
ORDER BY data, id;

-- name: wpisz_przeglad_lokalu :exec
-- Przegląd instalacji, czujnika albo gaśnicy z datą następnego i dokumentem.
INSERT INTO lokal_krotkoterminowy_lokal_przeglady (rodzaj, data, nastepny, dokument_id)
VALUES (:rodzaj, :data, :nastepny, :dokument_id);
