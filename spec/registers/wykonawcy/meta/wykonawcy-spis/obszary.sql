-- name: obszary_wykonawcow :many
-- Bieżące obszary pracy wszystkich wykonawców; Koordynator wybiera z nich wykonawcę dla lokalu.
SELECT *
FROM wykonawcy_spis_obszary
WHERE do IS NULL
ORDER BY obszar, wykonawca_id;

-- name: wpisz_obszar_wykonawcy :exec
-- Obszar pracy po decyzji o przyjęciu albo zmianie danych.
INSERT INTO wykonawcy_spis_obszary (wykonawca_id, obszar, dostepnosc, czas_dojazdu_do_usterki, od,
  decyzja_id)
VALUES (:wykonawca_id, :obszar, :dostepnosc, :czas_dojazdu_do_usterki, :od, :decyzja_id);

-- name: zakoncz_obszar_wykonawcy :exec
-- Obszar, w którym wykonawca już nie pracuje, po decyzji o zmianie danych albo wykreśleniu.
-- param zakonczenie_decyzja_id: string
UPDATE wykonawcy_spis_obszary
SET do = :do,
  zakonczenie_decyzja_id = :zakonczenie_decyzja_id
WHERE id = :id;
