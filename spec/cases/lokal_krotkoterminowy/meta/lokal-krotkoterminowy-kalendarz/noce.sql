-- name: noce :many
-- Kalendarz lokalu: noce w kolejności dat.
SELECT *
FROM lokal_krotkoterminowy_kalendarz_noce
ORDER BY data;

-- name: wpisz_noc :exec
-- Pierwszy wpis nocy w kalendarzu: wolna, zarezerwowana dla pobytu, na prace, dla właściciela albo
-- po końcu umowy.
INSERT INTO lokal_krotkoterminowy_kalendarz_noce (data, stan, pobyt_id, zlecenie_id, zgoda_id,
  zmieniono)
VALUES (:data, :stan, :pobyt_id, :zlecenie_id, :zgoda_id, :zmieniono);

-- name: zmien_noc :exec
-- Nowy stan nocy; zajętą noc najpierw się zwalnia (stan wolna).
UPDATE lokal_krotkoterminowy_kalendarz_noce
SET stan = :stan,
  pobyt_id = :pobyt_id,
  zlecenie_id = :zlecenie_id,
  zgoda_id = :zgoda_id,
  zmieniono = :zmieniono
WHERE data = :data;
