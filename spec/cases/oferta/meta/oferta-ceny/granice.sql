-- name: granice_negocjacji :many
-- Granice negocjacji od najstarszych; nie wychodzą poza teczkę oferty (R1).
SELECT *
FROM oferta_ceny_granice
ORDER BY dotyczy, rowid;

-- name: granice_negocjacji_biezace :many
-- Granice negocjacji, których nie zastąpiły nowsze.
SELECT *
FROM oferta_ceny_granice AS granice
WHERE NOT EXISTS (
  SELECT 1
  FROM oferta_ceny_granice AS nowsze
  WHERE nowsze.zastepuje_id = granice.id
)
ORDER BY granice.dotyczy;

-- name: wpisz_granice_negocjacji :exec
-- Najniższa cena albo czynsz, które klient przyjmie, z warunkami i jego zgodą; nowe granice
-- zastępują obowiązujące.
INSERT INTO oferta_ceny_granice (id, dotyczy, najnizsza_kwota, warunki, zgoda_id, zastepuje_id)
VALUES (:id, :dotyczy, :najnizsza_kwota, :warunki, :zgoda_id, :zastepuje_id);
