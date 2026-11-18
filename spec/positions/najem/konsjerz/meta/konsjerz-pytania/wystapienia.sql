-- name: dopisz_wystapienie_pytania :exec
-- Gość zadał pytanie, na które Konsjerż ma odpowiedź.
INSERT INTO konsjerz_pytania_wystapienia (pytanie_id, zadano)
VALUES (:pytanie_id, :zadano);

-- name: odnotuj_czy_odpowiedz_pomogla :exec
-- Konsjerż wie już, czy odpowiedź pomogła gościowi.
-- param pomogla: boolean
UPDATE konsjerz_pytania_wystapienia
SET pomogla = :pomogla
WHERE id = :id;
