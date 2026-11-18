-- name: dzialki :many
-- Działki ewidencyjne wszystkich nieruchomości teczki.
SELECT *
FROM sprawa_nieruchomosci_dzialki
ORDER BY nieruchomosc_id, obreb, numer_dzialki;

-- name: wpisz_dzialke :exec
-- Działka ewidencyjna nieruchomości.
INSERT INTO sprawa_nieruchomosci_dzialki (nieruchomosc_id, obreb, numer_dzialki, powierzchnia)
VALUES (:nieruchomosc_id, :obreb, :numer_dzialki, :powierzchnia);

-- name: zmien_dzialke :exec
-- Poprawka obrębu, numeru albo powierzchni działki.
UPDATE sprawa_nieruchomosci_dzialki
SET obreb = :obreb,
  numer_dzialki = :numer_dzialki,
  powierzchnia = :powierzchnia
WHERE id = :id;

-- name: wykresl_dzialke :exec
-- Poprawka pomyłki: działka, która nie należy do nieruchomości (D14).
DELETE FROM sprawa_nieruchomosci_dzialki
WHERE id = :id;
