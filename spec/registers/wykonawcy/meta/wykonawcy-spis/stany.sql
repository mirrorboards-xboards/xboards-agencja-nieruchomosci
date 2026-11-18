-- name: stany_wykonawcy :many
-- Historia stanu wykonawcy z powodami i decyzjami.
SELECT *
FROM wykonawcy_spis_stany
WHERE wykonawca_id = :wykonawca_id
ORDER BY id;

-- name: zmien_stan_wykonawcy :exec
-- Wstrzymanie, przywrócenie albo wykreślenie po decyzji człowieka tego rodzaju.
INSERT INTO wykonawcy_spis_stany (wykonawca_id, stan, od, powod, decyzja_id)
VALUES (:wykonawca_id, :stan, :od, :powod, :decyzja_id);
