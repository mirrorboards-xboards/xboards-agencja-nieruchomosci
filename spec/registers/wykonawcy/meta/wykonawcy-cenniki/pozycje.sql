-- name: pozycje_cennika :many
-- Pozycje jednej wersji cennika.
SELECT *
FROM wykonawcy_cenniki_pozycje
WHERE wersja_id = :wersja_id
ORDER BY id;

-- name: dopisz_pozycje_cennika :exec
-- Usługa albo dopłata w projekcie wersji, z ceną w groszach.
INSERT INTO wykonawcy_cenniki_pozycje (wersja_id, usluga, doplata, jednostka, cena, z_vat, warunki)
VALUES (:wersja_id, :usluga, :doplata, :jednostka, :cena, :z_vat, :warunki);

-- name: popraw_pozycje_cennika :exec
-- Poprawka pozycji, dopóki wersja jest projektem.
UPDATE wykonawcy_cenniki_pozycje
SET usluga = :usluga,
  doplata = :doplata,
  jednostka = :jednostka,
  cena = :cena,
  z_vat = :z_vat,
  warunki = :warunki
WHERE id = :id;
