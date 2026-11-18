-- name: czynnosci_umocowan :many
-- Czynności w zakresie wszystkich umocowań teczki.
SELECT *
FROM sprawa_umocowania_czynnosci
ORDER BY umocowanie_id, czynnosc;

-- name: dopisz_czynnosc_umocowania :exec
-- Czynność, do której umocowanie upoważnia, nazwana jak w typie sprawy.
INSERT INTO sprawa_umocowania_czynnosci (umocowanie_id, czynnosc)
VALUES (:umocowanie_id, :czynnosc);
