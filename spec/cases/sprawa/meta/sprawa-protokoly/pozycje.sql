-- name: pozycje_protokolow :many
-- Pozycje wszystkich protokołów teczki w kolejności wpisania.
SELECT *
FROM sprawa_protokoly_pozycje
ORDER BY protokol_id, id;

-- name: dopisz_pozycje_protokolu :exec
-- Licznik, rzecz, klucze, pomieszczenie albo uwaga w projekcie protokołu.
INSERT INTO sprawa_protokoly_pozycje (protokol_id, rodzaj, nazwa, numer, opis_stanu, zdjecie_sciezka,
  zdjecie_skrot)
VALUES (:protokol_id, :rodzaj, :nazwa, :numer, :opis_stanu, :zdjecie_sciezka, :zdjecie_skrot);

-- name: popraw_pozycje_protokolu :exec
-- Poprawka pozycji, dopóki protokół jest projektem.
UPDATE sprawa_protokoly_pozycje
SET nazwa = :nazwa,
  numer = :numer,
  opis_stanu = :opis_stanu,
  zdjecie_sciezka = :zdjecie_sciezka,
  zdjecie_skrot = :zdjecie_skrot
WHERE id = :id;
