-- name: weryfikacje_identyfikacji :many
-- Weryfikacje jednej identyfikacji, najnowsze pierwsze.
SELECT *
FROM rejestr_kontroli_sprawdzenia_weryfikacje
WHERE identyfikacja_id = :identyfikacja_id
ORDER BY id DESC;

-- name: wpisz_weryfikacje :exec
-- Weryfikacja danych identyfikacyjnych z podstawą i wynikiem.
INSERT INTO rejestr_kontroli_sprawdzenia_weryfikacje (identyfikacja_id, podstawa, wynik, opis,
  zrodlo, raport_odnosnik, sprawdzil, sprawdzono, wpisal, wpisano, aktualne, poprzedni_id)
VALUES (:identyfikacja_id, :podstawa, :wynik, :opis, :zrodlo, :raport_odnosnik, :sprawdzil,
  :sprawdzono, :wpisal, :wpisano, 1, :poprzedni_id);
