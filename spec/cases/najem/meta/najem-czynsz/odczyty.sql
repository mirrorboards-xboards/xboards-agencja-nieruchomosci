-- name: odczyty_licznikow :many
-- Odczyty liczników w kolejności chwili.
SELECT *
FROM najem_czynsz_odczyty
ORDER BY licznik, chwila, id;

-- name: wpisz_odczyt_licznika :exec
-- Odczyt licznika ze zdjęciem, porównany przez Opiekuna z poprzednim.
INSERT INTO najem_czynsz_odczyty (najem_id, licznik, numer, chwila, stan, zdjecie_sciezka,
  zdjecie_skrot)
VALUES (:najem_id, :licznik, :numer, :chwila, :stan, :zdjecie_sciezka, :zdjecie_skrot);
