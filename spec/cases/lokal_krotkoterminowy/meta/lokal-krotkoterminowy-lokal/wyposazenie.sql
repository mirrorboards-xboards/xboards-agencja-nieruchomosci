-- name: wyposazenie_lokalu :many
-- Wyposażenie biura w lokalu w kolejności wpisania, także wycofane.
SELECT *
FROM lokal_krotkoterminowy_lokal_wyposazenie
ORDER BY id;

-- name: wpisz_wyposazenie_lokalu :exec
-- Rzecz kupiona albo wymieniona przez biuro, z kosztem i datą.
INSERT INTO lokal_krotkoterminowy_lokal_wyposazenie (rzecz, od, koszt, zgoda_id)
VALUES (:rzecz, :od, :koszt, :zgoda_id);

-- name: wycofaj_wyposazenie_lokalu :exec
-- Rzecz zużyta, zniszczona albo wymieniona: dzień końca i powód.
-- param do_dnia: string
-- param powod: string
UPDATE lokal_krotkoterminowy_lokal_wyposazenie
SET do_dnia = :do_dnia,
  powod = :powod
WHERE id = :id;
