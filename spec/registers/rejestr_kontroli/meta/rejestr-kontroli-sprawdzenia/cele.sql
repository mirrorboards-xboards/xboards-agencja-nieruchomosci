-- name: cele_stosunkow_klienta :many
-- Cel i charakter stosunków z klientem, najnowsze pierwsze.
SELECT *
FROM rejestr_kontroli_sprawdzenia_cele
WHERE klient_id = :klient_id
ORDER BY id DESC;

-- name: wpisz_cel_stosunkow :exec
-- Cel i zamierzony charakter stosunków gospodarczych.
INSERT INTO rejestr_kontroli_sprawdzenia_cele (klient_id, cel, charakter, zrodlo, raport_odnosnik,
  sprawdzil, sprawdzono, wpisal, wpisano, aktualne, poprzedni_id)
VALUES (:klient_id, :cel, :charakter, :zrodlo, :raport_odnosnik, :sprawdzil, :sprawdzono, :wpisal,
  :wpisano, 1, :poprzedni_id);
