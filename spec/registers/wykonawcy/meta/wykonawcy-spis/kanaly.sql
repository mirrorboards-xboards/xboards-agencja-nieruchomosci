-- name: kanaly_wykonawcow :many
-- Bieżące kanały zleceń wszystkich wykonawców.
SELECT *
FROM wykonawcy_spis_kanaly
WHERE do IS NULL
ORDER BY wykonawca_id, id;

-- name: wpisz_kanal_wykonawcy :exec
-- Kanał zleceń po decyzji o przyjęciu albo zmianie danych.
INSERT INTO wykonawcy_spis_kanaly (wykonawca_id, rodzaj, adres, od, decyzja_id)
VALUES (:wykonawca_id, :rodzaj, :adres, :od, :decyzja_id);

-- name: zakoncz_kanal_wykonawcy :exec
-- Kanał, którego wykonawca już nie używa, po decyzji o zmianie danych albo wykreśleniu.
-- param zakonczenie_decyzja_id: string
UPDATE wykonawcy_spis_kanaly
SET do = :do,
  zakonczenie_decyzja_id = :zakonczenie_decyzja_id
WHERE id = :id;
