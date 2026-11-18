-- name: kontrole_organow :many
-- Kontrole wykonywania obowiązków, najnowsze pierwsze.
SELECT *
FROM rejestr_kontroli_zawiadomienia_kontrole
ORDER BY od DESC, id;

-- name: wpisz_kontrole_organu :exec
-- Kontrola z protokołem i zaleceniami.
INSERT INTO rejestr_kontroli_zawiadomienia_kontrole (organ, od, do, protokol_sciezka, protokol_skrot,
  zalecenia, wpisal, wpisano)
VALUES (:organ, :od, :do, :protokol_sciezka, :protokol_skrot, :zalecenia, :wpisal, :wpisano);
