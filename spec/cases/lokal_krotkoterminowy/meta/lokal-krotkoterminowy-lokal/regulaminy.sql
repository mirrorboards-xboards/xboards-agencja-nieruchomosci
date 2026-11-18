-- name: regulaminy_lokalu :many
-- Regulaminy lokalu w kolejności obowiązywania; ostatni jest bieżący.
SELECT *
FROM lokal_krotkoterminowy_lokal_regulaminy
ORDER BY od, rowid;

-- name: wpisz_regulamin_lokalu :exec
-- Regulamin lokalu; każdy po pierwszym wskazuje zatwierdzoną decyzję o zmianie. Identyfikator
-- (UUID) nadaje aplikacja, bo wskazuje go decyzja.
INSERT INTO lokal_krotkoterminowy_lokal_regulaminy (id, dokument_id, od, decyzja_id)
VALUES (:id, :dokument_id, :od, :decyzja_id);
