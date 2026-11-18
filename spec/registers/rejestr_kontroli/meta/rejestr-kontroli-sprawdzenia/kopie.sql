-- name: kopie_identyfikacji :many
-- Kopie dokumentów tożsamości jednej identyfikacji.
SELECT *
FROM rejestr_kontroli_sprawdzenia_kopie
WHERE identyfikacja_id = :identyfikacja_id
ORDER BY id DESC;

-- name: wpisz_kopie_dokumentu :exec
-- Kopia dokumentu tożsamości ze skrótem, dołączona przez Kontrolera do raportu.
INSERT INTO rejestr_kontroli_sprawdzenia_kopie (identyfikacja_id, plik_sciezka, plik_skrot, zrodlo,
  raport_odnosnik, sprawdzil, sprawdzono, wpisal, wpisano)
VALUES (:identyfikacja_id, :plik_sciezka, :plik_skrot, :zrodlo, :raport_odnosnik, :sprawdzil,
  :sprawdzono, :wpisal, :wpisano);
