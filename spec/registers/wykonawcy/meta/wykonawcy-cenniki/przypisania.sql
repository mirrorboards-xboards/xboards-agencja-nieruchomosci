-- name: przypisania_cennikow :many
-- Bieżące przypisania cenników do wykonawców.
SELECT *
FROM wykonawcy_cenniki_przypisania
WHERE do IS NULL
ORDER BY wykonawca_id, cennik_id;

-- name: przypisz_cennik :exec
-- Wykonawca pracuje według cennika od dnia, po decyzji o przypisaniu.
INSERT INTO wykonawcy_cenniki_przypisania (cennik_id, wykonawca_id, od, decyzja_id)
VALUES (:cennik_id, :wykonawca_id, :od, :decyzja_id);

-- name: zakoncz_przypisanie_cennika :exec
-- Koniec przypisania po decyzji o przypisaniu, np. gdy wykonawca przechodzi na inny cennik.
-- param zakonczenie_decyzja_id: string
UPDATE wykonawcy_cenniki_przypisania
SET do = :do,
  zakonczenie_decyzja_id = :zakonczenie_decyzja_id
WHERE id = :id;
