-- name: zakres_umow :many
-- Czynności, na które pozwalają umowy teczki.
SELECT *
FROM sprawa_umowy_zakres
ORDER BY umowa_id, czynnosc;

-- name: dopisz_czynnosc_umowy :exec
-- Czynność, na którą umowa pozwala biuru, nazwana jak w typie sprawy; tylko w projekcie umowy.
INSERT INTO sprawa_umowy_zakres (umowa_id, czynnosc)
VALUES (:umowa_id, :czynnosc);
