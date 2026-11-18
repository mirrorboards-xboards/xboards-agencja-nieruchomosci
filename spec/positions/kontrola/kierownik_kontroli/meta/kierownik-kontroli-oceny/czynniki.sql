-- name: czynniki_oceny :many
-- Czynniki jednego projektu według obszaru.
SELECT *
FROM kierownik_kontroli_oceny_czynniki
WHERE ocena_id = :ocena_id
ORDER BY obszar, id;

-- name: dopisz_czynnik :exec
-- Nowy czynnik ryzyka w projekcie roboczym.
INSERT INTO kierownik_kontroli_oceny_czynniki (ocena_id, obszar, czynnik, poziom, srodki, zrodlo)
VALUES (:ocena_id, :obszar, :czynnik, :poziom, :srodki, :zrodlo);

-- name: zmien_czynnik :exec
-- Nowy opis, poziom, środki albo źródło czynnika w projekcie roboczym.
UPDATE kierownik_kontroli_oceny_czynniki
SET obszar = :obszar,
  czynnik = :czynnik,
  poziom = :poziom,
  srodki = :srodki,
  zrodlo = :zrodlo
WHERE id = :id;
