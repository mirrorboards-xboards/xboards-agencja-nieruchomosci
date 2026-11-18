-- name: strony_umow :many
-- Strony wszystkich umów teczki.
SELECT *
FROM sprawa_umowy_strony
ORDER BY umowa_id, id;

-- name: dopisz_strone_umowy :exec
-- Klient, który jest stroną projektu umowy.
INSERT INTO sprawa_umowy_strony (umowa_id, uczestnik_id)
VALUES (:umowa_id, :uczestnik_id);
