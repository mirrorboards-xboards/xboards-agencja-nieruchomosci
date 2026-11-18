-- name: rozliczenia_wplat :many
-- Które należności pokrywa każda wpłata.
SELECT *
FROM najem_czynsz_rozliczenia
ORDER BY wplata_id, id;

-- name: rozlicz_wplate :exec
-- Część wpłaty na jedną należność, wyliczona przez program.
INSERT INTO najem_czynsz_rozliczenia (wplata_id, naleznosc_id, kwota)
VALUES (:wplata_id, :naleznosc_id, :kwota);
